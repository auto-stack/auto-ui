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
  return root.children;
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

function checkedWidgetAction(context, event, acceptsInputValue = false, typesOverride = event.types) {
  let message = event.message;
  for (let owner = context; owner !== null; owner = owner.parent) {
    if (Object.hasOwn(owner.definition.handlers, message)) {
      if (owner.definition.contextHandler === message) {
        throw new Error('ace.widget.context: only the host can deliver widget context');
      }
      const handler = owner.definition.handlers[message];
      const inputValue = acceptsInputValue && handler.types.length !== 0;
      const types = inputValue ? ['str'] : typesOverride;
      if (handler.types.length !== types.length ||
          handler.types.some((type, index) => widgetScalarType(type) !== types[index])) {
        throw new Error(`ace.widget.payload: mismatched arguments for '${message}'`);
      }
      return { path: owner.path, handler, types, values: event.values, inputValue };
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

function withWidgetContext(context, model) {
  const message = context.definition.contextHandler;
  if (message === null) return model;
  const width = Math.max(0, Math.min(2147483647, Math.round(globalThis.innerWidth || 0)));
  return context.definition.handlers[message].run(
    ...widgetArguments({ ...context, model }), widgetScalar('str', context.mountToken), width);
}

function checkedOutboundEvents(context, model) {
  const cells = model?.[checkedOutboundField]?.items ?? [];
  const events = [];
  for (let index = 0; index < cells.length;) {
    const message = a2x_str_utf8(cells[index++]);
    if (!Object.hasOwn(context.definition.outbound, message)) {
      throw new Error(`ace.widget.outbound: undeclared message '${message}'`);
    }
    const types = context.definition.outbound[message].map(widgetScalarType);
    if (index + types.length > cells.length) {
      throw new Error(`ace.widget.outbound: incomplete payload for '${message}'`);
    }
    const values = cells.slice(index, index + types.length).map(a2x_str_utf8);
    index += types.length;
    events.push({ message, types, values });
  }
  return events;
}

function sameWidgetValue(left, right) {
  const pairs = [[left, right]];
  while (pairs.length !== 0) {
    const [before, after] = pairs.pop();
    if (Object.is(before, after)) continue;
    if (before instanceof A2xOwnedStr && after instanceof A2xOwnedStr) {
      if (a2x_str_utf8(before) !== a2x_str_utf8(after)) return false;
    } else if (before instanceof A2xList && after instanceof A2xList) {
      if (before.items.length !== after.items.length) return false;
      for (const [index, item] of before.items.entries()) pairs.push([item, after.items[index]]);
    } else if (before && after && Object.getPrototypeOf(before) === Object.prototype &&
               Object.getPrototypeOf(after) === Object.prototype) {
      const fields = Object.keys(before);
      if (fields.length !== Object.keys(after).length) return false;
      for (const field of fields) {
        if (!Object.hasOwn(after, field)) return false;
        pairs.push([before[field], after[field]]);
      }
    } else {
      return false;
    }
  }
  return true;
}

function collectWidgetFocus(context, model, requests) {
  for (const target of model?.[checkedFocusField]?.items ?? []) {
    requests.push({ path: context.path, target: a2x_str_utf8(target) });
  }
}

function focusTargetId(target) {
  if ((target.startsWith("[id='") && target.endsWith("']")) ||
      (target.startsWith('[id="') && target.endsWith('"]'))) return target.slice(5, -2);
  if (target.startsWith('#')) return target.slice(1);
  return target;
}

function runWidgetWatches(previous, context, focus) {
  const value = (instance, source) => typeof source === 'number' ? instance.inputs[source] : instance.model[source];
  const active = context.definition.watches.filter(watch => previous === undefined
    ? watch.immediate
    : watch.sources.some(source => !sameWidgetValue(value(previous, source), value(context, source))));
  for (const watch of active) {
    context.model = watch.run(...widgetArguments(context));
    collectWidgetFocus(context, context.model, focus);
  }
}

const checkedDomEvents = {
  onmousemove: 'onMousemove',
  onmouseenter: 'onMouseenter',
  onmouseleave: 'onMouseleave',
  onclick: 'onClick',
  ondblclick: 'onDblclick',
  oncontextmenu: 'onContextmenu',
  onfocus: 'onFocus',
  onblur: 'onBlur',
  onkeydown: 'onKeydown',
  oninput: 'onInput',
  onenter: 'onKeydown',
  onerror: 'onError',
};

const checkedDomKeys = {
  left: 'ArrowLeft',
  right: 'ArrowRight',
  up: 'ArrowUp',
  down: 'ArrowDown',
  home: 'Home',
  end: 'End',
  enter: 'Enter',
  space: ' ',
  escape: 'Escape',
};

function checkedDomBinding(binding) {
  const [event, ...modifiers] = binding.toLowerCase().split('.');
  const flags = new Set(modifiers);
  const keys = modifiers.filter(modifier => Object.hasOwn(checkedDomKeys, modifier));
  if (!Object.hasOwn(checkedDomEvents, event) || flags.size !== modifiers.length ||
      keys.length > 1 || (keys.length === 1 && event !== 'onkeydown') ||
      modifiers.some(modifier => !keys.includes(modifier) && !['self', 'stop', 'prevent'].includes(modifier))) {
    throw new Error(`ace.widget.event: unsupported binding '${binding}'`);
  }
  return { property: checkedDomEvents[event], key: event === 'onenter' ? 'Enter' : checkedDomKeys[keys[0]], flags };
}

function nativeWidgetNode(element, children, click, close, listeners, focusMarker) {
  const { tag, properties } = element;
  const props = { key: properties.get('key') };
  if (focusMarker !== null) props['data-auto-checked-focus'] = focusMarker;
  for (const name of ['id', 'title', 'aria-label', 'aria-expanded', 'role', 'tabindex']) {
    if (properties.has(name)) props[name] = properties.get(name);
  }
  const layout = {
    col: 'flex flex-col',
    row: 'flex flex-row',
    grid: 'grid',
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
    'alert-dialog-header': 'flex flex-col gap-2',
    'alert-dialog-title': 'text-lg font-semibold',
    'alert-dialog-description': 'text-sm text-muted-foreground',
    'alert-dialog-footer': 'flex justify-end gap-2',
    'alert-dialog-cancel': 'rounded-md border border-input px-4 py-2 text-sm',
    'alert-dialog-action': 'rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground',
  };
  const buttonClasses = tag === 'button' && properties.get('variant') === 'outline'
    ? 'inline-flex items-center justify-center rounded-md border border-input bg-background px-3 py-1 text-sm hover:bg-accent'
    : undefined;
  props.class = [layout[tag], buttonClasses, properties.get('style'), properties.get('class')];
  props.style = logicalBoxStyle(properties.get('coords'), props.class);
  for (const [property, callbacks] of listeners) props[property] = callbacks;
  const label = properties.get('text');
  if (label !== undefined && children.length === 0) children = [label];
  if (tag === 'button' && properties.has('icon')) {
    const icon = properties.get('icon');
    if (!Object.hasOwn(aceIcons, icon)) throw new Error(`ace.widget.icon: unresolved icon '${icon}'`);
    if (children.length === 0 && props['aria-label'] === undefined) props['aria-label'] = icon;
    children = [h(aceIcons[icon], { class: 'size-4', 'aria-hidden': 'true' }), ...children];
  }
  if (click !== null) {
    if ((children.every(child => child === '') || tag === 'mouse-area') && props['aria-label'] === undefined) {
      props['aria-label'] = [click.message, ...click.values].join(' ');
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
  if (tag === 'input') return h('input', {
    ...props, type: properties.get('type') ?? 'text', value: properties.get('value') ?? '',
    placeholder: properties.get('placeholder'), disabled: properties.get('disabled') === 'true',
  });
  if (tag === 'image_surface') return h('img', {
    ...props, key: properties.get('cache_identity') ?? props.key, src: properties.get('src'),
    alt: properties.get('alt') ?? '', style: { objectFit: properties.get('fit') },
  });
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
  if (tag === 'alert-dialog') return h('dialog', {
    ...props, role: 'alertdialog', 'aria-modal': 'true',
    class: ['m-auto w-full max-w-lg rounded-lg border bg-background p-5 text-foreground shadow-lg backdrop:bg-black/50', props.class],
    onVnodeMounted: ({ el: dialog }) => { if (!dialog.open) dialog.showModal(); },
    onCancel: event => event.preventDefault(),
  }, children);
  if (tag === 'sidebar_menu_button' && properties.get('active') === 'true') {
    props.class.push('bg-accent text-accent-foreground');
    props['aria-current'] = 'location';
  }
  if (['button', 'sidebar_menu_button', 'alert-dialog-cancel', 'alert-dialog-action'].includes(tag) ||
      ((tag === 'mouse-area' || tag === 'row') && click !== null)) {
    return h('button', { type: 'button', ...props, disabled: properties.get('disabled') === 'true' }, children);
  }
  const native = {
    col: 'div', row: 'div', grid: 'div', 'mouse-area': 'div', scroll: 'div', 'preview-card': 'section', text: 'span',
    sidebar_provider: 'div', sidebar_content: 'div', sidebar_group: 'div',
    sidebar_group_label: 'div', sidebar_group_content: 'div', sidebar_menu: 'ul',
    sidebar_menu_item: 'li',
    'table-header': 'thead', 'table-body': 'tbody', 'table-row': 'tr',
    'table-head': 'th', 'table-cell': 'td',
    'alert-dialog-content': 'div', 'alert-dialog-header': 'div', 'alert-dialog-title': 'h2',
    'alert-dialog-description': 'p', 'alert-dialog-footer': 'div',
  };
  if (Object.hasOwn(native, tag)) return h(native[tag], props, children);
  if (/^(div|span|p|strong|em|small|h[1-6]|section|article|header|aside|main|table|thead|tbody|tr|th|td|ul|ol|li)$/.test(tag)) {
    return h(tag, props, children);
  }
  throw new Error(`ace.widget.view: unsupported element '${tag}'`);
}

function createCheckedWidgetView(root, fileDirectory, fileMutation, preferences = [], fileOpen = null) {
  const rendered = shallowRef(null);
  const pointerSamples = new WeakMap();
  let current = null;
  let savedPreferences = [];
  let active = true;
  let mounted = false;
  const scheduled = new Map();
  const focusScopeKey = Symbol.for('auto.checked-widget-focus-scope');
  const focusScope = globalThis[focusScopeKey] = (globalThis[focusScopeKey] ?? 0) + 1;
  let nextFocusMarker = 0;
  let mountNamespace;
  let nextMountToken = 0;
  let directoryRequest = null;
  let mutationRequest = null;

  function draw(changes = new Map(), focus = []) {
    const frame = { models: new Map(), timerOwners: new Map(), focus, focusTargets: new Map() };
    const building = new Set();

    function widget(node, inputs, path, parent, channels) {
      if (building.has(node)) throw new Error('ace.widget.view: recursive component');
      building.add(node);
      const definition = aceComponents[node];
      const context = { definition, inputs, path, parent, channels, model: null };
      if (definition.init !== null) {
        if (frame.models.has(path)) throw new Error('ace.widget.view: duplicate instance key');
        const previous = current?.models.get(path);
        if (definition.contextHandler !== null) {
          mountNamespace ??= globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
          context.mountToken = previous?.mountToken ?? `vue-${mountNamespace}-${nextMountToken++}`;
        }
        if (changes.has(path)) context.model = changes.get(path);
        else if (previous !== undefined) context.model = previous.model;
        else {
          context.model = withWidgetContext(context, definition.init(...widgetInputs(inputs)));
          if (path === 'root') {
            for (const [key, , restore] of preferences) {
              let value;
              try { value = globalThis.localStorage?.getItem(key); } catch {}
              if (value != null) {
                context.model = definition.handlers[restore].run(...widgetArguments(context), widgetScalar('str', value));
                collectWidgetFocus(context, context.model, frame.focus);
              }
            }
          }
        }
        const inputsChanged = previous === undefined ||
          inputs.some((value, index) => !sameWidgetValue(value, previous.inputs[index]));
        if (inputsChanged) {
          if (Object.hasOwn(definition.handlers, 'Init')) {
            const handler = definition.handlers.Init;
            if (handler.types.length !== 0) throw new Error('ace.widget.init: Init must take no payload');
            context.model = handler.run(...widgetArguments(context));
            collectWidgetFocus(context, context.model, frame.focus);
            if (checkedOutboundEvents(context, context.model).some(event => Object.hasOwn(channels, event.message))) {
              throw new Error('ace.widget.init: outbound events require a mounted parent');
            }
          }
        }
        if (definition.watches.length !== 0) runWidgetWatches(previous, context, frame.focus);
        frame.models.set(path, context);
      }
      if (definition.timers.length !== 0) {
        if (frame.timerOwners.has(path)) throw new Error('ace.widget.timer: duplicate instance key');
        frame.timerOwners.set(path, context);
      }
      const children = render(checkedViewItems(definition.view(...widgetArguments(context))), context, path);
      building.delete(node);
      return children;
    }

    function render(items, owner, parentPath) {
      const siteOccurrences = new Map();
      return items.map(element => {
        if (typeof element === 'string') return element;
        if (element.tag === 'svg') return checkedSvgNode(element);
        if (element.tag === 'alert-dialog' && element.properties.get('open') !== 'true') return null;
        const key = element.properties.get('key');
        const occurrence = siteOccurrences.get(element.site) ?? 0;
        siteOccurrences.set(element.site, occurrence + 1);
        const path = parentPath + JSON.stringify([element.site, key ?? occurrence]);
        const site = owner.definition.sites[element.site];
        if (site !== undefined && site.tag === element.tag) {
          if (element.children.length !== 0) throw new Error('ace.widget.view: unexpected widget slot content');
          const inputs = site.props.map(prop => prop.get !== undefined
            ? prop.get(...(prop.defaulted ? [] : widgetArguments(owner)))
            : widgetScalar(widgetScalarType(prop.type), element.properties.get(prop.name)));
          return h(Fragment, { key: path }, widget(site.node, inputs, path, owner, site.channels));
        }
        let click = null;
        let close = null;
        const bindings = new Set();
        const listeners = new Map();
        for (const event of element.events) {
          const kind = event.kind.toLowerCase();
          if (bindings.has(kind)) throw new Error(`ace.widget.event: duplicate binding '${event.kind}'`);
          bindings.add(kind);
          const acceptsInputValue = element.tag === 'input' && kind === 'oninput' && event.types.length === 0;
          const pointerMove = element.tag === 'mouse-area' && kind === 'onmousemove' && event.types.length === 0;
          const action = checkedWidgetAction(owner, event, acceptsInputValue,
            pointerMove ? ['f32', 'f32'] : event.types);
          const extent = pointerMove ? element.properties.get('coords') : undefined;
          const dimensions = widgetExtent(extent);
          if (extent !== undefined && dimensions === null) {
            throw new Error('ace.widget.pointer: coords must have a finite positive WxH extent');
          }
          if (element.tag === 'drawer' && kind === 'onclose') {
            close = { run: domEvent => {
              domEvent.stopPropagation();
              dispatch(frame, action);
            } };
            continue;
          }
          const { property, key, flags } = checkedDomBinding(kind);
          if (property === 'onClick') click ??= event;
          const callbacks = listeners.get(property) ?? [];
          callbacks.push(domEvent => {
            if (!active || current !== frame || element.properties.get('disabled') === 'true' ||
                (key !== undefined && domEvent.key !== key) ||
                (flags.has('self') && domEvent.target !== domEvent.currentTarget)) return;
            if (pointerMove) {
              const [width, height] = dimensions ?? [];
              const values = acePointerSample(domEvent, pointerSamples, width, height);
              if (values !== undefined) {
                domEvent.stopPropagation();
                dispatch(frame, { ...action, values });
              }
              return;
            }
            if (kind === 'onmouseleave') pointerSamples.delete(domEvent.currentTarget);
            if (property === 'onClick' || flags.has('stop') || kind === 'onmouseenter' || kind === 'onmouseleave') {
              domEvent.stopPropagation();
            }
            if (flags.has('prevent')) domEvent.preventDefault();
            if (action.inputValue) {
              dispatch(frame, { ...action, values: [domEvent.currentTarget.value] });
            } else {
              dispatch(frame, action);
            }
          });
          listeners.set(property, callbacks);
        }
        return nativeWidgetNode(element, render(element.children, owner, path), click, close, listeners,
          registerFocusTarget(frame, owner, element.properties.get('id')));
      });
    }

    frame.view = widget(root, [], 'root', null, {});
    return frame;
  }

  function registerFocusTarget(frame, owner, id) {
    if (!owner.definition.focus || !id) return null;
    let targets = frame.focusTargets.get(owner.path);
    if (targets === undefined) {
      targets = new Map();
      frame.focusTargets.set(owner.path, targets);
    }
    const existing = targets.get(id);
    if (existing !== undefined) {
      existing.duplicate = true;
      return null;
    }
    // These VNodes are assembled before Vue renders. A host marker lets
    // the committed frame find its element without relying on a render ref.
    const target = { marker: `${focusScope}-${nextFocusMarker++}`, duplicate: false };
    targets.set(id, target);
    return target.marker;
  }

  function applyFrameFocus(frame) {
    if (!active || current !== frame) return;
    for (let index = frame.focus.length - 1; index >= 0; index -= 1) {
      const { path, target } = frame.focus[index];
      const match = frame.focusTargets.get(path)?.get(focusTargetId(target));
      if (match === undefined || match.duplicate) continue;
      const element = globalThis.document?.querySelector(`[data-auto-checked-focus="${match.marker}"]`);
      if (!element || element.disabled || typeof element.focus !== 'function') continue;
      element.focus();
      break;
    }
    frame.focus.length = 0;
  }

  function keep(frame) {
    const models = frame === null ? [] : Array.from(frame.models.values(), instance => [instance.model, instance.inputs]);
    if (directoryRequest !== null) models.push(...directoryRequest.changes.values());
    a2x_widget_keep_models(models);
  }

  function publish(frame) {
    keep(frame);
    const model = frame.models.get('root')?.model;
    const nextPreferences = preferences.map(([, field]) => {
      const value = model[field];
      return typeof value === 'boolean' ? String(value) : a2x_str_utf8(value);
    });
    if (current !== null) {
      for (const [index, [key]] of preferences.entries()) {
        if (savedPreferences[index] !== nextPreferences[index]) {
          try { globalThis.localStorage?.setItem(key, nextPreferences[index]); } catch {}
        }
      }
      current.models.clear();
      current.timerOwners.clear();
      current.view = null;
    }
    savedPreferences = nextPreferences;
    current = frame;
    rendered.value = frame.view;
    if (frame.focus.length !== 0) void nextTick(() => applyFrameFocus(frame));
    if (mounted) {
      reconcileTimers(frame.timerOwners);
      startPendingMutation();
    }
  }

  function reconcileTimers(owners) {
    for (const [path, schedule] of scheduled) {
      if (owners.get(path)?.definition === schedule.definition) continue;
      for (const handle of schedule.handles) clearInterval(handle);
      scheduled.delete(path);
    }
    for (const [path, { definition }] of owners) {
      if (scheduled.has(path)) continue;
      const schedule = { definition, handles: [] };
      scheduled.set(path, schedule);
      for (const timer of definition.timers) {
        schedule.handles.push(setInterval(() => {
          const frame = current;
          const owner = frame?.timerOwners.get(path);
          if (!active || owner === undefined || scheduled.get(path) !== schedule) return;
          if (directoryRequest !== null && path === 'root') return;
          const action = checkedWidgetAction(owner, { message: timer.message, types: [], values: [] });
          const enabled = timer.condition === null ? null : () => timer.condition(...widgetArguments(owner));
          dispatch(frame, action, enabled);
        }, timer.interval));
      }
    }
  }

  function pendingDirectoryPath(path, model) {
    if (fileDirectory === null || path !== 'root' || !model.file_request_pending) return null;
    return a2x_str_utf8(model.file_request_path);
  }

  function cancelDirectoryRequest() {
    const request = directoryRequest;
    directoryRequest = null;
    request?.controller.abort();
  }

  async function finishDirectoryRequest(request, path) {
    let directory;
    try {
      directory = await fileDirectory.read(path, request.controller.signal, fileDirectory.includeExtension);
    } catch {
      directory = null;
    }
    if (directoryRequest !== request || !active || current !== request.frame) return;
    directoryRequest = null;
    if (fileDirectory.lease.aborted) {
      keep(current);
      return;
    }
    a2x_widget_begin_frame();
    try {
      const entries = a2x_list_new(fileDirectory.entryTag);
      const ownedText = value => a2x_str_to_owned(a2x_str_lit(value));
      for (const entry of directory?.entries ?? []) {
        const row = { ...entry, name: ownedText(entry.name), path: ownedText(entry.path) };
        if (fileDirectory.includeExtension) row.extension = ownedText(entry.extension);
        a2x_list_push(entries, row);
      }
      const listing = { ok: directory !== null, path: ownedText(directory?.path ?? ''), entries };
      const instance = current.models.get('root');
      const handler = instance.definition.handlers[fileDirectory.reply];
      const changes = request.changes;
      const model = handler.run(...widgetArguments({ ...instance, model: changes.get('root') }), listing);
      if (pendingDirectoryPath('root', model) !== null) {
        throw new Error('ace.widget.file-reply: directory reply must clear the pending request');
      }
      changes.set('root', model);
      collectWidgetFocus(instance, model, request.focus);
      publish(draw(changes, request.focus));
    } catch (error) {
      keep(current);
      throw error;
    }
    deliverMutationReply();
  }

  function startDirectoryRequest(frame, changes, path, focus = []) {
    cancelDirectoryRequest();
    if (fileDirectory.lease.aborted) {
      keep(current);
      return;
    }
    const request = { frame, changes, focus, controller: new AbortController() };
    directoryRequest = request;
    keep(current);
    void finishDirectoryRequest(request, path);
  }

  function startPendingMutation() {
    if (fileMutation === null || fileMutation.lease.aborted || mutationRequest !== null) return;
    const model = current.models.get('root').model;
    if (!model.file_mutation_pending) return;
    const request = { controller: new AbortController(), result: null };
    const operation = a2x_str_utf8(model.file_mutation_operation);
    const path = a2x_str_utf8(model.file_mutation_path);
    const target = a2x_str_utf8(model.file_mutation_target);
    mutationRequest = request;
    void finishMutationRequest(request, operation, path, target);
  }

  async function finishMutationRequest(request, operation, path, target) {
    let ok;
    try {
      ok = await fileMutation.apply(operation, path, target, request.controller.signal);
    } catch {
      ok = false;
    }
    if (!active || mutationRequest !== request || fileMutation.lease.aborted) return;
    request.result = ok;
    deliverMutationReply();
  }

  function deliverMutationReply() {
    if (mutationRequest === null || mutationRequest.result === null || directoryRequest !== null) return;
    if (!active || fileMutation.lease.aborted) return;
    // A pending read owns a newer staged model. Once it settles, deliver
    // to the published root so its state survives navigation and redraws.
    const instance = current.models.get('root');
    a2x_widget_begin_frame();
    try {
      const handler = instance.definition.handlers[fileMutation.reply];
      const model = handler.run(...widgetArguments(instance), mutationRequest.result);
      if (model.file_mutation_pending) {
        throw new Error('ace.widget.file-reply: mutation reply must clear the pending request');
      }
      const path = pendingDirectoryPath('root', model);
      const changes = new Map([['root', model]]);
      const focus = [];
      collectWidgetFocus(instance, model, focus);
      const next = draw(changes, focus);
      // Commit the reply before a refresh can be replaced or canceled.
      publish(next);
      mutationRequest = null;
      if (path !== null) startDirectoryRequest(next, changes, path);
    } catch (error) {
      keep(current);
      throw error;
    }
  }

  function applyWidgetAction(frame, action, changes, focus, budget) {
    if (budget.remaining-- === 0) throw new Error('ace.widget.outbound: transition limit exceeded');
    const instance = frame.models.get(action.path);
    const previous = withWidgetContext(instance, changes.get(action.path) ?? instance.model);
    const payload = action.values.map((value, index) => widgetScalar(action.types[index], value));
    const model = action.handler.run(...widgetArguments({ ...instance, model: previous }), ...payload);
    changes.set(action.path, model);
    collectWidgetFocus(instance, model, focus);
    for (const event of checkedOutboundEvents(instance, model)) {
      if (instance.parent === null || !Object.hasOwn(instance.channels, event.message)) continue;
      const message = instance.channels[event.message];
      const target = checkedWidgetAction(instance.parent, { ...event, message });
      applyWidgetAction(frame, target, changes, focus, budget);
    }
  }

  function dispatch(frame, action, enabled = null) {
    if (!active || current !== frame) return;
    a2x_widget_begin_frame();
    try {
      if (enabled !== null && !enabled()) {
        keep(current);
        return;
      }
      const changes = new Map();
      const focus = [];
      applyWidgetAction(frame, action, changes, focus, { remaining: 4096 });
      let rootModel = changes.get('root');
      if (fileOpen !== null && rootModel?.file_open_pending) {
        const instance = frame.models.get('root');
        const handler = instance.definition.handlers[fileOpen.reply];
        rootModel = handler.run(...widgetArguments({ ...instance, model: rootModel }), fileOpen.unavailable);
        if (rootModel.file_open_pending) {
          throw new Error('ace.widget.file-reply: open reply must clear the pending request');
        }
        changes.set('root', rootModel);
        collectWidgetFocus(instance, rootModel, focus);
      }
      const path = rootModel === undefined ? null : pendingDirectoryPath('root', rootModel);
      if (path !== null) {
        startDirectoryRequest(frame, changes, path, focus);
      } else {
        const next = draw(changes, focus);
        cancelDirectoryRequest();
        publish(next);
        deliverMutationReply();
      }
    } catch (error) {
      keep(current);
      throw error;
    }
  }

  onMounted(() => {
    if (current === null) return;
    mounted = true;
    reconcileTimers(current.timerOwners);
    if (fileDirectory !== null) {
      const model = current.models.get('root').model;
      const path = pendingDirectoryPath('root', model);
      if (path !== null) startDirectoryRequest(current, new Map([['root', model]]), path);
    }
    startPendingMutation();
  });

  onBeforeUnmount(() => {
    active = false;
    mounted = false;
    cancelDirectoryRequest();
    mutationRequest?.controller.abort();
    mutationRequest = null;
    reconcileTimers(new Map());
    current?.models.clear();
    current?.timerOwners.clear();
    if (current !== null) current.view = null;
    current = null;
    rendered.value = null;
    keep(null);
  });

  a2x_widget_begin_frame();
  try {
    publish(draw());
  } catch (error) {
    keep(null);
    throw error;
  }
  return { name: 'AcePackageView', setup: () => () => rendered.value };
}
