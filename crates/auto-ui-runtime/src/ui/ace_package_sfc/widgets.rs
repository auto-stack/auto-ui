use super::{escape_attr, escape_html, expr_to_template_js, render_items, TmplCtx};
use crate::ui::ace_widget_props::{
    element_children, expression, flag, literal, self_field, square_style, style,
};
use ace_core::source::age_component::ComponentViewElement;
use ace_core::source::ast::ExprKind;

pub(super) fn render(
    element: &ComponentViewElement,
    ctx: &mut TmplCtx<'_>,
) -> Result<bool, String> {
    match element.tag.as_str() {
        "accordion" => accordion(element, ctx)?,
        "collapsible" => collapsible(element, ctx)?,
        "pagination" => pagination(element, ctx)?,
        "center" => {
            let classes = style(element);
            let margin = if classes.split_whitespace().any(|name| name.starts_with("max-w-")) {
                " mx-auto"
            } else {
                ""
            };
            let height = if classes.split_whitespace().any(|name| name.starts_with("h-")) {
                ""
            } else {
                " h-full"
            };
            ctx.push_str(&format!(
                "<div class=\"flex flex-col items-center justify-center{height}{margin} {}\">",
                escape_attr(&classes)
            ));
            render_items(&element.items, ctx)?;
            ctx.push_str("</div>");
        }
        "grid" => grid(element, ctx)?,
        "spacer" => {
            let classes = style(element);
            let classes = if classes.is_empty() { "flex-1" } else { &classes };
            ctx.push_str(&format!("<div class=\"{}\"></div>", escape_attr(classes)));
        }
        "carousel" => carousel(element, ctx, "AceCarousel")?,
        "carousel-content" | "carousel_content" => carousel(element, ctx, "AceCarouselContent")?,
        "carousel-item" | "carousel_item" => carousel(element, ctx, "AceCarouselItem")?,
        "carousel-previous" | "carousel_previous" | "carousel-prev" | "carousel_prev" => {
            carousel(element, ctx, "AceCarouselPrevious")?
        }
        "carousel-next" | "carousel_next" => carousel(element, ctx, "AceCarouselNext")?,
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

fn grid(element: &ComponentViewElement, ctx: &mut TmplCtx<'_>) -> Result<(), String> {
    let mut classes = vec!["grid".to_owned()];
    let mut dynamic = Vec::new();
    match expression(element, "cols").or_else(|| expression(element, "columns")) {
        Some(expr) => match &expr.kind {
            ExprKind::IntLit(cols) => classes.push(format!("grid-cols-{}", (*cols).max(1))),
            _ => dynamic.push(format!(
                "gridTemplateColumns: 'repeat(' + Math.max(1, {}) + ', minmax(0, 1fr))'",
                expr_to_template_js(expr)?
            )),
        },
        None => classes.push("grid-cols-1".into()),
    }
    if let Some(expr) = expression(element, "gap") {
        match &expr.kind {
            ExprKind::IntLit(gap) if *gap >= 0 && gap % 4 == 0 => {
                classes.push(format!("gap-{}", gap / 4));
            }
            ExprKind::IntLit(gap) => classes.push(format!("gap-[{}px]", (*gap).max(0))),
            _ => dynamic.push(format!("gap: Math.max(0, {}) + 'px'", expr_to_template_js(expr)?)),
        }
    }
    classes.push(style(element));
    ctx.push_str(&format!("<div class=\"{}\"", escape_attr(classes.join(" ").trim())));
    if !dynamic.is_empty() {
        ctx.push_str(&format!(" :style=\"{{ {} }}\"", escape_attr(&dynamic.join(", "))));
    }
    ctx.push_str(">");
    render_items(&element.items, ctx)?;
    ctx.push_str("</div>");
    Ok(())
}

fn carousel(element: &ComponentViewElement, ctx: &mut TmplCtx<'_>, tag: &str) -> Result<(), String> {
    ctx.carousel_used = true;
    ctx.push_str(&format!("<{tag}"));
    if element.tag == "carousel" {
        if let Some(orientation) = expression(element, "orientation") {
            match &orientation.kind {
                ExprKind::StrLit(value) => {
                    ctx.push_str(&format!(" orientation=\"{}\"", escape_attr(value)))
                }
                _ => ctx.push_str(&format!(
                    " :orientation=\"{}\"",
                    escape_attr(&expr_to_template_js(orientation)?)
                )),
            }
        }
        let mut options = Vec::new();
        if let Some(opts) = expression(element, "opts") {
            options.push(format!("...({})", expr_to_template_js(opts)?));
        }
        if let Some(align) = expression(element, "align") {
            let value = match &align.kind {
                ExprKind::StrLit(value) if matches!(value.as_str(), "start" | "center" | "end") => {
                    format!("'{value}'")
                }
                ExprKind::StrLit(_) => {
                    return Err("source.age.sfc-carousel: align must be start, center, or end".into());
                }
                _ => expr_to_template_js(align)?,
            };
            options.push(format!("align: {value}"));
        }
        if let Some(looping) = expression(element, "loop") {
            let value = match &looping.kind {
                ExprKind::StrLit(_) => flag(element, "loop").to_string(),
                _ => expr_to_template_js(looping)?,
            };
            options.push(format!("loop: {value}"));
        }
        if !options.is_empty() {
            ctx.push_str(&format!(" :opts=\"{{ {} }}\"", escape_attr(&options.join(", "))));
        }
    }
    let classes = style(element);
    if !classes.is_empty() {
        ctx.push_str(&format!(" class=\"{}\"", escape_attr(&classes)));
    }
    ctx.push_str(">");
    render_label(element, ctx)?;
    ctx.push_str(&format!("</{tag}>"));
    Ok(())
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
