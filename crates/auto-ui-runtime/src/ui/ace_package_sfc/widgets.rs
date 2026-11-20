use super::{escape_attr, escape_html, expr_to_template_js, render_items, TmplCtx};
use crate::ui::ace_widget_props::{
    element_children, expression, flag, literal, self_field, square_style, style,
};
use ace_core::source::age_component::ComponentViewElement;

pub(super) fn render(
    element: &ComponentViewElement,
    ctx: &mut TmplCtx<'_>,
) -> Result<bool, String> {
    match element.tag.as_str() {
        "accordion" => accordion(element, ctx)?,
        "collapsible" => collapsible(element, ctx)?,
        "pagination" => pagination(element, ctx)?,
        "preview-card" => {
            ctx.push_str("<section");
            if let Some(id) = literal(element, "id") {
                ctx.push_str(&format!(" id=\"{}\"", escape_attr(id)));
            }
            ctx.push_str(&format!(
                " class=\"w-full min-w-0 border rounded-md p-4 {}\">",
                escape_attr(&style(element))
            ));
            render_items(&element.items, ctx)?;
            ctx.push_str("</section>");
        }
        "square" => {
            ctx.push_str(&format!(
                "<div class=\"flex items-center justify-center {}\">",
                escape_attr(&square_style(element))
            ));
            if element_children(element).next().is_none() {
                if let Some(text) = literal(element, "text") {
                    ctx.push_str(&escape_html(text));
                }
            }
            render_items(&element.items, ctx)?;
            ctx.push_str("</div>");
        }
        "codeblock" => {
            let code = literal(element, "code").unwrap_or_default();
            ctx.push_str(&format!("<pre><code>{}</code></pre>", escape_html(code)));
        }
        "separator" => ctx.push_str("<hr />"),
        _ => return Ok(false),
    }
    Ok(true)
}

fn widget_ref(ctx: &mut TmplCtx<'_>, value: &str) -> String {
    ctx.widget_count += 1;
    let name = format!("__ace_widget_{}", ctx.widget_count);
    ctx.widget_state
        .push_str(&format!("const {name} = ref({value})\n"));
    name
}

fn render_label(element: &ComponentViewElement, ctx: &mut TmplCtx<'_>) -> Result<(), String> {
    if let Some(label) = &element.label {
        ctx.push_str(&format!(
            "{{{{ {} }}}}",
            escape_html(&expr_to_template_js(label)?)
        ));
    }
    render_items(&element.items, ctx)
}

fn accordion(element: &ComponentViewElement, ctx: &mut TmplCtx<'_>) -> Result<(), String> {
    let multiple = literal(element, "type") == Some("multiple");
    let selected = widget_ref(ctx, if multiple { "[]" } else { "null" });
    let collapsible = flag(element, "collapsible");
    ctx.push_str(&format!("<div class=\"{}\">", escape_attr(&style(element))));
    for (index, item) in element_children(element)
        .filter(|child| child.tag == "accordion-item")
        .enumerate()
    {
        let active = if multiple {
            format!("{selected}.includes({index})")
        } else {
            format!("{selected} === {index}")
        };
        let toggle = if multiple {
            format!("{selected} = {active} ? {selected}.filter(i => i !== {index}) : [...{selected}, {index}]")
        } else if collapsible {
            format!("{selected} = {active} ? null : {index}")
        } else {
            format!("{selected} = {index}")
        };
        let panel = format!("{selected}_panel_{index}");
        ctx.push_str("<div>");
        for child in element_children(item) {
            match child.tag.as_str() {
                "accordion-trigger" => {
                    ctx.push_str(&format!("<button type=\"button\" aria-controls=\"{panel}\" :aria-expanded=\"{active}\" @click=\"{toggle}\" class=\"{}\">", escape_attr(&style(child))));
                    render_label(child, ctx)?;
                    ctx.push_str("</button>");
                }
                "accordion-content" => {
                    ctx.push_str(&format!(
                        "<div id=\"{panel}\" v-show=\"{active}\" class=\"{}\">",
                        escape_attr(&style(child))
                    ));
                    render_label(child, ctx)?;
                    ctx.push_str("</div>");
                }
                _ => {}
            }
        }
        ctx.push_str("</div>");
    }
    ctx.push_str("</div>");
    Ok(())
}

fn collapsible(element: &ComponentViewElement, ctx: &mut TmplCtx<'_>) -> Result<(), String> {
    let local = widget_ref(
        ctx,
        if flag(element, "open") {
            "true"
        } else {
            "false"
        },
    );
    let open = expression(element, "open")
        .and_then(self_field)
        .unwrap_or(&local);
    let disabled = flag(element, "disabled");
    let panel = format!("{local}_panel");
    ctx.push_str(&format!(
        "<div class=\"{}\" :data-state=\"{open} ? 'open' : 'closed'\">",
        escape_attr(&style(element))
    ));
    for child in element_children(element) {
        match child.tag.as_str() {
            "collapsible-trigger" => {
                let disabled = if disabled { " disabled" } else { "" };
                ctx.push_str(&format!("<button type=\"button\" aria-controls=\"{panel}\" :aria-expanded=\"{open}\" :data-state=\"{open} ? 'open' : 'closed'\" @click=\"{open} = !{open}\"{disabled} class=\"{}\">", escape_attr(&style(child))));
                render_label(child, ctx)?;
                ctx.push_str("</button>");
            }
            "collapsible-content" => {
                ctx.push_str(&format!(
                    "<div id=\"{panel}\" v-show=\"{open}\" class=\"{}\">",
                    escape_attr(&style(child))
                ));
                render_label(child, ctx)?;
                ctx.push_str("</div>");
            }
            _ => {}
        }
    }
    ctx.push_str("</div>");
    Ok(())
}

fn pagination(element: &ComponentViewElement, ctx: &mut TmplCtx<'_>) -> Result<(), String> {
    let number = |name, default: &str| {
        expression(element, name)
            .map(expr_to_template_js)
            .transpose()
            .map(|v| v.unwrap_or_else(|| default.to_owned()))
    };
    let total = number("total", "0")?;
    let per_page = number("per_page", "10")?;
    let last = escape_attr(&format!(
        "Math.max(1, Math.ceil(Number({total}) / Math.max(1, Number({per_page}))))"
    ));
    let page = widget_ref(ctx, &number("page", "1")?);
    ctx.push_str(&format!(
        "<nav aria-label=\"Pagination\" :data-current-page=\"{page}\" class=\"{}\">",
        escape_attr(&style(element))
    ));
    for list in element_children(element).filter(|child| child.tag == "pagination-list") {
        ctx.push_str("<ul class=\"flex items-center gap-2\">");
        for child in element_children(list) {
            match child.tag.as_str() {
                "pagination-previous" => ctx.push_str(&format!("<li><button type=\"button\" :disabled=\"{page} <= 1\" @click=\"{page} = Math.max(1, {page} - 1)\">Previous</button></li>")),
                "pagination-next" => ctx.push_str(&format!("<li><button type=\"button\" :disabled=\"{page} >= {last}\" @click=\"{page} = Math.min({last}, {page} + 1)\">Next</button></li>")),
                "pagination-ellipsis" => ctx.push_str("<li aria-hidden=\"true\">…</li>"),
                "pagination-item" => {
                    let Some(value) = expression(child, "value") else { continue };
                    let value = escape_attr(&expr_to_template_js(value)?);
                    ctx.push_str(&format!("<li><button type=\"button\" :aria-current=\"{page} === Number({value}) ? 'page' : undefined\" :class=\"{{ 'font-bold underline': {page} === Number({value}) }}\" @click=\"{page} = Math.max(1, Math.min({last}, Number({value})))\">"));
                    render_label(child, ctx)?;
                    ctx.push_str("</button></li>");
                }
                _ => {}
            }
        }
        ctx.push_str("</ul>");
    }
    ctx.push_str("</nav>");
    Ok(())
}
