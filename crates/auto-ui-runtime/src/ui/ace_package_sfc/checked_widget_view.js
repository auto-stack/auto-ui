function checkedViewItems(tape) {
  const cells = tape.items;
  const root = { children: [] };
  const stack = [root];
  const timers = [];
  let index = 0;
  const next = () => {
    if (index === cells.length) throw new Error('ace.widget.view: truncated output');
    return a2x_str_utf8(cells[index++]);
  };
  while (index < cells.length) {
    const current = stack[stack.length - 1];
    const operation = next();
    if (operation === 'element') {
      const element = { tag: next(), site: next(), properties: new Map(), events: [], children: [] };
      current.children.push(element);
      stack.push(element);
    } else if (operation === 'text') {
      current.children.push(next());
    } else if (operation === 'timer' && stack.length === 1) {
      const message = next();
      const everyMs = Number(next());
      if (!Number.isInteger(everyMs) || everyMs < 1 || everyMs > 2147483647) {
        throw new Error('ace.widget.timer: invalid interval');
      }
      timers.push({ message, everyMs });
    } else if (operation === 'end') {
      if (stack.length <= 1 || stack.pop().tag !== next()) {
        throw new Error('ace.widget.view: unbalanced elements');
      }
    } else if (stack.length === 1) {
      throw new Error(`ace.widget.view: '${operation}' outside an element`);
    } else if (operation === 'property') {
      current.properties.set(next(), next());
    } else if (operation === 'label') {
      current.children.push(next());
    } else if (operation === 'event') {
      const event = { kind: next(), message: next(), types: [], values: [] };
      const count = Number(next());
      if (!Number.isInteger(count) || count < 0 || count * 2 > cells.length - index) {
        throw new Error('ace.widget.view: invalid event arity');
      }
      for (let argument = 0; argument < count; argument += 1) {
        event.types.push(widgetScalarType(next()));
        event.values.push(next());
      }
      current.events.push(event);
    } else {
      throw new Error(`ace.widget.view: unknown operation '${operation}'`);
    }
  }
  if (stack.length !== 1) throw new Error('ace.widget.view: incomplete elements');
  return { items: root.children, timers };
}

function widgetScalarType(type) {
  if (type === 'str' || type === 'Str') return 'str';
  if (type === 'int' || type === 'i32') return 'i32';
  if (type === 'bool') return 'bool';
  if (type === 'float' || type === 'f32') return 'f32';
  throw new Error(`ace.widget.payload: unsupported type '${type}'`);
}

function widgetScalar(type, value) {
  if (type === 'str' && typeof value === 'string') return a2x_str_to_owned(a2x_str_lit(value));
  if (type === 'bool' && (value === 'true' || value === 'false')) return value === 'true';
  if (type === 'f32' && typeof value === 'number' && Number.isFinite(value)) return Math.fround(value);
  if (type === 'i32' && /^-?\d+$/.test(value)) {
    const number = Number(value);
    if (Number.isInteger(number) && number >= -2147483648 && number <= 2147483647) return number;
  }
  throw new Error(`ace.widget.payload: invalid ${type} value`);
}

function widgetInputs(inputs) {
  // Checked scalar text parameters transfer ownership into each call.
  return inputs.map(value => value instanceof A2xOwnedStr ? a2x_str_clone(value) : value);
}

function widgetArguments(context) {
  const args = widgetInputs(context.inputs);
  if (context.definition.init !== null) args.push(context.model);
  return args;
}

function sameWidgetInput(left, right) {
  if (Object.is(left, right)) return true;
  if (left === null || right === null || typeof left !== 'object' || typeof right !== 'object') {
    return false;
  }
  if (left instanceof A2xOwnedStr || right instanceof A2xOwnedStr) {
    return left instanceof A2xOwnedStr && right instanceof A2xOwnedStr &&
      sameWidgetInput(left.as_bytes(), right.as_bytes());
  }
  if (left instanceof Uint8Array || right instanceof Uint8Array) {
    return left instanceof Uint8Array && right instanceof Uint8Array &&
      left.length === right.length && left.every((byte, index) => byte === right[index]);
  }
  if (left instanceof A2xList || right instanceof A2xList) {
    return left instanceof A2xList && right instanceof A2xList &&
      left.elemKind === right.elemKind && left.items.length === right.items.length &&
      left.items.every((value, index) => sameWidgetInput(value, right.items[index]));
  }
  if (left instanceof A2xVariant || right instanceof A2xVariant) {
    return left instanceof A2xVariant && right instanceof A2xVariant &&
      left.disc === right.disc && sameWidgetInput(left.payload, right.payload);
  }
  const keys = Object.keys(left);
  return keys.length === Object.keys(right).length &&
    keys.every(key => Object.hasOwn(right, key) && sameWidgetInput(left[key], right[key]));
}

function checkedWidgetAction(context, event, types = event.types) {
  let message = event.message;
  for (let owner = context; owner !== null; owner = owner.parent) {
    if (Object.hasOwn(owner.definition.handlers, message)) {
      const handler = owner.definition.handlers[message];
      if (handler.types.length !== types.length ||
          handler.types.some((type, index) => widgetScalarType(type) !== types[index])) {
        throw new Error(`ace.widget.payload: mismatched arguments for '${message}'`);
      }
      return { path: owner.path, handler, types, values: event.values };
    }
    if (!Object.hasOwn(owner.channels, message)) break;
    message = owner.channels[message];
  }
  throw new Error(`ace.widget.event: no checked receiver for '${event.message}'`);
}

function checkedSvgNode(element) {
  const { tag, properties, children } = element;
  if (!aceSvgTags.has(tag)) throw new Error(`source.age.sfc-svg: unsupported SVG tag '${tag}'`);
  if (element.events.length !== 0) throw new Error(`source.age.sfc-svg: unsupported event on SVG '${tag}'`);
  const props = {};
  if (tag === 'svg') props.xmlns = aceSvgNamespace;
  for (const [key, value] of properties) {
    if (key === 'xmlns' && value === aceSvgNamespace) continue;
    if (!aceSvgProperties.has(key)) throw new Error(`source.age.sfc-svg: unsupported '${key}' on SVG '${tag}'`);
    if (key !== 'style' && key !== 'text') props[key] = value;
  }
  props.class = [properties.get('class'), properties.get('style')];
  const textOnly = tag === 'text' || tag === 'title' || tag === 'desc';
  const content = children.map(child => typeof child !== 'string'
    ? checkedSvgNode(child)
    : textOnly ? child : h('text', child));
  const label = properties.get('text');
  if (content.length === 0 && label !== undefined) content.push(label);
  return h(tag, props, content);
}

function widgetExtent(coords) {
  if (coords === undefined) return null;
  const size = coords.split(/[xX]/).map(value => Math.fround(Number(value.trim())));
  return size.length === 2 && size.every(value => Number.isFinite(value) && value > 0) ? size : null;
}

function logicalBoxStyle(coords, classes) {
  const size = widgetExtent(coords);
  if (size === null) return undefined;
  const utilities = classes.filter(Boolean).join(' ').split(/\s+/);
  if (!utilities.includes('absolute')) return undefined;
  const properties = {};
  const mapping = { left: ['left', 0], right: ['right', 0], w: ['width', 0],
    top: ['top', 1], bottom: ['bottom', 1], h: ['height', 1] };
  for (const token of utilities) {
    const match = /^(left|right|top|bottom|w|h)-\[(-?(?:\d+(?:\.\d+)?|\.\d+))px\]$/.exec(token);
    if (match === null) continue;
    const [property, axis] = mapping[match[1]];
    const percent = Number(match[2]) / size[axis] * 100;
    if (Number.isFinite(percent)) properties[property] = percent + '%';
  }
  return properties;
}

function nativeWidgetNode(element, children, click, close, mouse) {
  const { tag, properties } = element;
  const props = { key: properties.get('key'), ...mouse };
  for (const name of ['id', 'title', 'aria-label', 'aria-expanded', 'role', 'tabindex']) {
    if (properties.has(name)) props[name] = properties.get(name);
  }
  const layout = {
    col: 'flex flex-col',
    row: 'flex flex-row',
    scroll: 'overflow-y-auto',
    'preview-card': 'w-full min-w-0 border rounded-md p-4',
    sidebar_provider: 'flex',
    sidebar_content: 'min-h-0 flex-1 overflow-auto',
    sidebar_group: 'relative flex w-full min-w-0 flex-col p-2',
    sidebar_group_label: 'flex h-8 items-center px-2 text-xs font-medium text-muted-foreground',
    sidebar_group_content: 'w-full text-sm',
    sidebar_menu: 'flex w-full min-w-0 flex-col gap-1',
    sidebar_menu_item: 'relative',
    sidebar_menu_button: 'flex w-full items-center gap-2 rounded-md p-2 text-sm hover:bg-accent [&>svg]:size-4',
  };
  const buttonClasses = tag === 'button' && properties.get('variant') === 'outline'
    ? 'inline-flex items-center justify-center rounded-md border border-input bg-background px-3 py-1 text-sm hover:bg-accent'
    : undefined;
  props.class = [layout[tag], buttonClasses, properties.get('style'), properties.get('class')];
  props.style = logicalBoxStyle(properties.get('coords'), props.class);
  const label = properties.get('text');
  if (label !== undefined && children.length === 0) children = [label];
  if (tag === 'button' && properties.has('icon')) {
    const icon = properties.get('icon');
    if (!Object.hasOwn(aceIcons, icon)) throw new Error(`ace.widget.icon: unresolved icon '${icon}'`);
    if (children.length === 0 && props['aria-label'] === undefined) props['aria-label'] = icon;
    children = [h(aceIcons[icon], { class: 'size-4', 'aria-hidden': 'true' }), ...children];
  }
  if (click !== null) {
    props.onClick = click.run;
    if ((children.every(child => child === '') || tag === 'mouse-area') && props['aria-label'] === undefined) {
      props['aria-label'] = [click.action.message, ...click.action.values].join(' ');
    }
  }
  if (tag === 'icon') {
    const name = properties.get('name');
    if (!Object.hasOwn(aceIcons, name)) throw new Error(`ace.widget.icon: unresolved icon '${name}'`);
    return h(aceIcons[name], { ...props, 'aria-hidden': 'true' });
  }
  if (tag === 'codeblock') return h('pre', props, [h('code', properties.get('code') ?? '')]);
  if (tag === 'separator') return h('hr', props);
  if (tag === 'checkbox') {
    if (props['aria-label'] === undefined && children.every(child => typeof child === 'string')) {
      const label = children.join(' ');
      if (label) props['aria-label'] = label;
    }
    return h('input', {
      ...props,
      type: 'checkbox',
      checked: properties.get('checked') === 'true',
      disabled: click === null,
    });
  }
  if (tag === 'outlet' && aceRouting?.outlet) {
    return h(aceRouting.outlet, props, { default: ({ Component, route }) => Component
      ? h(Component, { key: route.path })
      : h('pre', `ace.route.no-match: no route for ${route.path}`) });
  }
  if ((tag === 'link' || tag === 'sidebar_menu_button') && aceRouting && properties.has('to')) {
    const activeClass = tag === 'sidebar_menu_button' ? 'bg-accent text-accent-foreground' : 'router-link-active';
    return h(aceRouting.link, {
      ...props,
      to: properties.get('to'),
      activeClass: properties.get('exact') === 'true' ? 'router-link-active' : activeClass,
      exactActiveClass: activeClass,
    }, { default: () => children });
  }
  if (tag === 'drawer') {
    if (properties.get('open') !== 'true') return null;
    return h('div', { class: 'fixed inset-0 z-50 flex items-end' }, [
      close === null ? null : h('button', {
        type: 'button', class: 'absolute inset-0 bg-black/40', 'aria-label': 'Close drawer', onClick: close.run,
      }),
      h('section', {
        ...props, class: ['relative w-full rounded-t-lg border bg-background p-4', props.class], role: 'dialog',
        onKeydown: event => { if (event.key === 'Escape') close?.run(event); },
      }, children),
    ]);
  }
  if (tag === 'button' || ((tag === 'mouse-area' || tag === 'row') && click !== null)) {
    return h('button', { type: 'button', ...props }, children);
  }
  const native = {
    col: 'div', row: 'div', scroll: 'div', 'preview-card': 'section', text: 'span',
    'mouse-area': 'div', 'table-header': 'thead', 'table-body': 'tbody',
    'table-row': 'tr', 'table-head': 'th', 'table-cell': 'td',
    sidebar_provider: 'div', sidebar_content: 'div', sidebar_group: 'div',
    sidebar_group_label: 'div', sidebar_group_content: 'div', sidebar_menu: 'ul',
    sidebar_menu_item: 'li',
  };
  if (Object.hasOwn(native, tag)) return h(native[tag], props, children);
  if (/^(div|span|p|strong|em|small|h[1-6]|section|article|header|aside|main|table|thead|tbody|tr|th|td|ul|ol|li)$/.test(tag)) {
    return h(tag, props, children);
  }
  throw new Error(`ace.widget.view: unsupported element '${tag}'`);
}

function createCheckedWidgetView(root) {
  const rendered = shallowRef(null);
  const pointerSamples = new WeakMap();
  const scheduled = new Map();
  let current = null;
  let active = true;

  function draw(changedPath, changedModel) {
    const frame = { models: new Map(), timers: new Map() };
    const building = new Set();

    function widget(node, inputs, path, parent, channels) {
      if (building.has(node)) throw new Error('ace.widget.view: recursive component');
      building.add(node);
      const definition = aceComponents[node];
      const context = { definition, inputs, path, parent, channels, model: null };
      if (definition.init !== null) {
        if (frame.models.has(path)) throw new Error('ace.widget.view: duplicate instance key');
        const previous = current?.models.get(path);
        if (path === changedPath) context.model = changedModel;
        else if (previous !== undefined) context.model = previous.model;
        else context.model = definition.init(...widgetInputs(inputs));
        // Getters can return new owned copies of unchanged props. Re-run
        // Init over the current model only when the input values change.
        const changedInputs = previous === undefined ||
          inputs.some((value, index) => !sameWidgetInput(value, previous.inputs[index]));
        if (changedInputs && Object.hasOwn(definition.handlers, 'Init')) {
          const handler = definition.handlers.Init;
          if (handler.types.length !== 0) throw new Error('ace.widget.init: Init must take no payload');
          context.model = handler.run(...widgetArguments(context));
        }
        frame.models.set(path, { definition, inputs, model: context.model });
      }
      const view = checkedViewItems(definition.view(...widgetArguments(context)));
      for (const { message, everyMs } of view.timers) {
        const key = JSON.stringify([path, node, message, everyMs]);
        if (frame.timers.has(key)) throw new Error(`ace.widget.timer: duplicate '${message}'`);
        const action = checkedWidgetAction(context, { message, types: [], values: [] });
        frame.timers.set(key, { action, everyMs });
      }
      const children = render(view.items, context, path);
      building.delete(node);
      return children;
    }

    function render(items, owner, parentPath) {
      return items.map((element, index) => {
        if (typeof element === 'string') return element;
        if (element.tag === 'svg') return checkedSvgNode(element);
        const key = element.properties.get('key');
        const path = parentPath + JSON.stringify([element.site, key ?? index]);
        const site = owner.definition.sites[element.site];
        if (site !== undefined && site.tag === element.tag) {
          if (element.children.length !== 0) throw new Error('ace.widget.view: unexpected widget slot content');
          const inputs = site.props.map(prop => prop.get !== undefined
            ? prop.get(...(prop.defaulted ? [] : widgetArguments(owner)))
            : widgetScalar(widgetScalarType(prop.type), element.properties.get(prop.name)));
          return widget(site.node, inputs, path, owner, site.channels);
        }
        let click = null;
        let close = null;
        const mouse = {};
        for (const event of element.events) {
          const kind = event.kind.toLowerCase();
          const isMouseArea = element.tag === 'mouse-area';
          const move = isMouseArea && kind === 'onmousemove' && event.types.length === 0;
          const action = checkedWidgetAction(owner, event, move ? ['f32', 'f32'] : event.types);
          const callback = { action: event, run: (domEvent, values) => {
            domEvent.stopPropagation();
            dispatch(frame, action, values);
          } };
          if (kind === 'onclick' && click === null) click = callback;
          else if (element.tag === 'drawer' && kind === 'onclose' && close === null) close = callback;
          else if (move && mouse.onMousemove === undefined) {
            const extent = element.properties.get('coords');
            const dimensions = widgetExtent(extent);
            if (extent !== undefined && dimensions === null) {
              throw new Error('ace.widget.pointer: coords must have a finite positive WxH extent');
            }
            const [width, height] = dimensions ?? [];
            mouse.onMousemove = domEvent => {
              const values = acePointerSample(domEvent, pointerSamples, width, height);
              if (values !== undefined) callback.run(domEvent, values);
            };
          }
          else if (isMouseArea && kind === 'onmouseenter' && mouse.onMouseenter === undefined) {
            mouse.onMouseenter = callback.run;
          }
          else if (isMouseArea && kind === 'onmouseleave' && mouse.onMouseleave === undefined) {
            mouse.onMouseleave = domEvent => {
              pointerSamples.delete(domEvent.currentTarget);
              callback.run(domEvent);
            };
          }
          else throw new Error(`ace.widget.event: unsupported binding '${event.kind}'`);
        }
        return nativeWidgetNode(element, render(element.children, owner, path), click, close, mouse);
      });
    }

    frame.view = widget(root, [], 'root', null, {});
    return frame;
  }

  function keep(frame) {
    a2x_widget_keep_models(frame === null ? [] : Array.from(frame.models.values(), instance => [instance.model, instance.inputs]));
  }

  function publish(frame) {
    keep(frame);
    if (current !== null) {
      current.models.clear();
      current.view = null;
    }
    current = frame;
    rendered.value = frame.view;
    for (const [key, interval] of scheduled) {
      if (frame.timers.has(key)) continue;
      clearInterval(interval);
      scheduled.delete(key);
    }
    for (const [key, timer] of frame.timers) {
      if (scheduled.has(key)) continue;
      const interval = setInterval(() => {
        if (!active || scheduled.get(key) !== interval) return;
        const pending = current.timers.get(key);
        if (pending !== undefined) dispatch(current, pending.action);
      }, timer.everyMs);
      scheduled.set(key, interval);
    }
  }

  function dispatch(frame, action, values = action.values) {
    if (!active || current !== frame) return;
    a2x_widget_begin_frame();
    try {
      const instance = frame.models.get(action.path);
      const payload = values.map((value, index) => widgetScalar(action.types[index], value));
      const model = action.handler.run(...widgetArguments(instance), ...payload);
      publish(draw(action.path, model));
    } catch (error) {
      keep(current);
      throw error;
    }
  }

  onBeforeUnmount(() => {
    active = false;
    for (const interval of scheduled.values()) clearInterval(interval);
    scheduled.clear();
    current?.models.clear();
    if (current !== null) current.view = null;
    current = null;
    rendered.value = null;
    keep(null);
  });

  a2x_widget_begin_frame();
  try {
    publish(draw(null, null));
  } catch (error) {
    keep(null);
    throw error;
  }
  return { name: 'AcePackageView', setup: () => () => rendered.value };
}
