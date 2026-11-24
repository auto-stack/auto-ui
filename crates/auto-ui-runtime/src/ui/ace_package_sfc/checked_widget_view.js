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
    } else if (operation === 'image-surface-event') {
      const kind = next();
      const message = next();
      if (!isCheckedImageSurface(current.tag) || (kind !== 'onwheel' && kind !== 'onpan')) {
        throw new Error('ace.widget.view: invalid image surface event');
      }
      const types = kind === 'onwheel' ? ['f32'] : ['f32', 'f32', 'str'];
      current.events.push({ kind, message, types, values: [], imageInput: true });
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

function checkedCanvasTexts(model, name) {
  const list = model === null || model === undefined ? undefined : model[name];
  if (list === undefined || list === null || !Array.isArray(list.items)) return [];
  // 模型里的 owned Str 读取按转移语义记：先 clone 再解码（否则 use-after-free）。
  return list.items.map(item => (typeof item === 'string' ? item : a2x_str_utf8(a2x_str_clone(item))));
}

function checkedCanvasNumber(parts, index, fallback) {
  const value = Number(parts[index]);
  return Number.isFinite(value) ? value : fallback;
}

function checkedCanvasNodes(texts) {
  const nodes = [];
  for (const text of texts) {
    if (!text) continue;
    const parts = text.split(',').map(part => part.trim());
    if (parts.length < 5 || !parts[0]) continue;
    const x = Number(parts[1]);
    const y = Number(parts[2]);
    if (!Number.isFinite(x) || !Number.isFinite(y)) continue;
    nodes.push({ id: parts[0], x, y, shape: parts[3].toLowerCase() === 'rect' ? 'rect' : 'circle',
      color: parts[4] || '#3b82f6', v: parts.slice(5) });
  }
  return nodes;
}

function checkedCanvasHit(nodes, x, y) {
  for (let index = nodes.length - 1; index >= 0; index -= 1) {
    const node = nodes[index];
    if (node.shape === 'rect') {
      const w = checkedCanvasNumber(node.v, 0, 40);
      const h = checkedCanvasNumber(node.v, 1, w);
      if (Math.abs(x - node.x) <= w / 2 && Math.abs(y - node.y) <= h / 2) return node.id;
    } else {
      const r = checkedCanvasNumber(node.v, 0, 16);
      const dx = x - node.x;
      const dy = y - node.y;
      if (dx * dx + dy * dy <= r * r) return node.id;
    }
  }
  return null;
}

function checkedCanvasDraw(canvas, scene) {
  const context = canvas.getContext('2d');
  if (context === null) return;
  const width = canvas.width;
  const height = canvas.height;
  context.clearRect(0, 0, width, height);
  context.fillStyle = scene.clear;
  context.fillRect(0, 0, width, height);
  context.lineCap = 'round';
  context.lineJoin = 'round';
  for (const stroke of scene.strokes) {
    context.strokeStyle = stroke.color;
    context.lineWidth = stroke.width;
    context.beginPath();
    stroke.points.forEach(([x, y], index) => {
      if (index === 0) context.moveTo(x, y);
      else context.lineTo(x, y);
    });
    context.stroke();
  }
  for (const edge of scene.edges) {
    context.strokeStyle = edge.color;
    context.lineWidth = edge.width;
    context.beginPath();
    context.moveTo(edge.x1, edge.y1);
    context.lineTo(edge.x2, edge.y2);
    context.stroke();
  }
  for (const node of scene.nodes) {
    context.fillStyle = node.color;
    if (node.shape === 'rect') {
      const w = checkedCanvasNumber(node.v, 0, 40);
      const h = checkedCanvasNumber(node.v, 1, w);
      context.fillRect(node.x - w / 2, node.y - h / 2, w, h);
    } else {
      const r = checkedCanvasNumber(node.v, 0, 16);
      context.beginPath();
      context.arc(node.x, node.y, r, 0, Math.PI * 2);
      context.fill();
    }
  }
  for (const label of scene.labels) {
    context.fillStyle = label.color;
    context.font = `${label.size}px sans-serif`;
    context.fillText(label.text, label.x, label.y);
  }
}

function checkedCanvasScene(model, name) {
  const texts = suffix => checkedCanvasTexts(model, `${name}_${suffix}`);
  const meta = texts('meta');
  const strokes = [];
  texts('pts').forEach((text, index) => {
    if (!text) return;
    const points = text.split('|').map(pair => pair.split(',')).filter(pair => pair.length === 2)
      .map(([x, y]) => [Number(x), Number(y)]).filter(([x, y]) => Number.isFinite(x) && Number.isFinite(y));
    if (points.length === 0) return;
    const pieces = (meta[index] ?? '').split(',');
    strokes.push({ points, color: pieces[0] || '#111827',
      width: Number(pieces[1]) > 0 ? Number(pieces[1]) : 3, eraser: pieces[2] === '1' });
  });
  const edges = texts('edges').map(text => text.split(',').map(part => part.trim()))
    .filter(parts => parts.length >= 4 && parts.slice(0, 4).every(part => Number.isFinite(Number(part))))
    .map(parts => ({ x1: Number(parts[0]), y1: Number(parts[1]), x2: Number(parts[2]), y2: Number(parts[3]),
      color: parts[4] || '#9ca3af', width: Number(parts[5]) > 0 ? Number(parts[5]) : 2 }));
  const labels = texts('labels').map(text => {
    const parts = text.split(',');
    if (parts.length < 3) return null;
    const x = Number(parts[0]);
    const y = Number(parts[1]);
    if (!Number.isFinite(x) || !Number.isFinite(y)) return null;
    return { x, y, text: parts.slice(2, parts.length - 2).join(',') || parts[2],
      color: parts.length >= 5 ? parts[parts.length - 2] : '#111827',
      size: parts.length >= 5 && Number(parts[parts.length - 1]) > 0 ? Number(parts[parts.length - 1]) : 12 };
  }).filter(label => label !== null);
  return { nodes: checkedCanvasNodes(texts('nodes')), strokes, edges, labels };
}

function checkedCanvasNode(element, owner, frame, path, dispatch) {
  const size = widgetExtent(element.properties.get('coords')) ?? [320, 320];
  const [width, height] = size;
  const sceneName = element.properties.get('scene');
  const clear = element.properties.get('clear') ?? '#ffffff';
  const classes = element.properties.get('style') ?? '';
  const scene = sceneName === undefined ? { nodes: [], strokes: [], edges: [], labels: [] }
    : { clear, ...checkedCanvasScene(owner.model, sceneName) };
  const paint = ({ el }) => checkedCanvasDraw(el, scene);
  const hitEvent = element.events.find(event => event.kind.toLowerCase() === 'onhit');
  const onHit = domEvent => {
    if (hitEvent === undefined) return;
    const rect = domEvent.currentTarget.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return;
    const x = (domEvent.clientX - rect.left) * (width / rect.width);
    const y = (domEvent.clientY - rect.top) * (height / rect.height);
    const id = checkedCanvasHit(scene.nodes, x, y);
    if (id === null) return;
    const action = checkedWidgetAction(owner, { ...hitEvent, types: ['str'], values: [id] }, true);
    dispatch(frame, action);
  };
  return h('canvas', {
    key: path,
    class: classes,
    width,
    height,
    style: { width: `${width}px`, height: `${height}px` },
    onVnodeMounted: paint,
    onVnodeUpdated: paint,
    onClick: onHit,
  });
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

function* outboundWidgetActions(context, model) {
  for (const event of checkedOutboundEvents(context, model)) {
    if (context.parent === null || !Object.hasOwn(context.channels, event.message)) continue;
    const message = context.channels[event.message];
    yield checkedWidgetAction(context.parent, { ...event, message });
  }
}

function sameWidgetValue(left, right) {
  const pairs = [[left, right]];
  let visited;
  while (pairs.length !== 0) {
    const [before, after] = pairs.pop();
    if (Object.is(before, after)) continue;
    if (before === null || after === null || typeof before !== 'object' || typeof after !== 'object') return false;
    let matches = (visited ??= new WeakMap()).get(before);
    if (matches === undefined) visited.set(before, matches = new WeakSet());
    if (matches.has(after)) continue;
    matches.add(after);
    if (before instanceof A2xOwnedStr && after instanceof A2xOwnedStr) {
      if (a2x_str_utf8(before) !== a2x_str_utf8(after)) return false;
    } else if (before instanceof A2xList && after instanceof A2xList) {
      if (before.items.length !== after.items.length) return false;
      for (const [index, item] of before.items.entries()) pairs.push([item, after.items[index]]);
    } else if (before instanceof A2xDyn && after instanceof A2xDyn) {
      pairs.push([before.type, after.type], [before.value, after.value]);
    } else if (before instanceof A2xObject && after instanceof A2xObject) {
      pairs.push([before.entries, after.entries]);
    } else if (before instanceof Uint8Array && after instanceof Uint8Array) {
      if (before.length !== after.length || !before.every((byte, index) => byte === after[index])) return false;
    } else if (Array.isArray(before) && Array.isArray(after)) {
      if (before.length !== after.length) return false;
      for (const [index, item] of before.entries()) pairs.push([item, after[index]]);
    } else if (Object.getPrototypeOf(before) === Object.prototype &&
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
  onchange: 'onChange',
  onenter: 'onKeydown',
  onload: 'onLoad',
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

function isCheckedImageSurface(tag) {
  return tag === 'image_surface' || tag === 'image-surface' || tag === 'imagesurface';
}

function checkedImageNode(properties, props, path) {
  const source = properties.get('src');
  const cacheIdentity = properties.get('cache_identity');
  const fit = properties.get('fit') ?? 'contain';
  const numeric = (key, fallback) => {
    const value = Number(properties.get(key));
    return Number.isFinite(value) ? value : fallback;
  };
  const zoom = Math.min(64, Math.max(0.05, numeric('zoom', 1)));
  const width = fit === 'contain' || fit === 'cover' || fit === 'width' ? '100%' : 'auto';
  const height = fit === 'contain' || fit === 'cover' ? '100%' : 'auto';
  const { class: classes, style: viewportStyle, ...imageProps } = props;
  if (cacheIdentity === undefined) {
    imageProps.src = source || undefined;
  } else {
    const binding = { value: [source, cacheIdentity] };
    const update = ({ el }) => vAutoVersionedImage.updated(el, binding);
    imageProps.onVnodeMounted = update;
    imageProps.onVnodeUpdated = update;
    imageProps.onVnodeBeforeUnmount = ({ el }) => vAutoVersionedImage.unmounted(el);
  }
  return h('div', {
    key: path,
    class: ['relative overflow-hidden', classes],
    style: { display: 'grid', placeItems: 'center', overflow: 'hidden', ...viewportStyle },
  }, [h('img', {
    ...imageProps,
    key: JSON.stringify([source, cacheIdentity]),
    alt: properties.get('alt') ?? '',
    draggable: false,
    style: {
      width, height, maxWidth: 'none', maxHeight: 'none', userSelect: 'none',
      touchAction: props.onPointerdown ? 'none' : undefined,
      objectFit: fit === 'cover' ? 'cover' : 'contain',
      filter: properties.get('filter'),
      transform: `translate(${numeric('offset_x', 0)}px, ${numeric('offset_y', 0)}px) rotate(${numeric('rotation', 0)}deg) scale(${zoom})`,
    },
  })]);
}

function checkedPanListeners(samples, enabled, run) {
  function end(event) {
    const element = event.currentTarget;
    const sample = samples.get(element);
    if (sample?.id !== event.pointerId) return;
    samples.delete(element);
    if (element.hasPointerCapture(event.pointerId)) element.releasePointerCapture(event.pointerId);
    run([0, 0, 'end']);
  }
  return [
    ['onPointerdown', event => {
      const element = event.currentTarget;
      if (!enabled() || event.button !== 0 || samples.has(element)) return;
      element.setPointerCapture(event.pointerId);
      samples.set(element, { id: event.pointerId, x: event.clientX, y: event.clientY });
      run([0, 0, 'start']);
    }],
    ['onPointermove', event => {
      const sample = samples.get(event.currentTarget);
      if (sample?.id !== event.pointerId) return;
      const dx = event.clientX - sample.x;
      const dy = event.clientY - sample.y;
      sample.x = event.clientX;
      sample.y = event.clientY;
      if (dx !== 0 || dy !== 0) run([dx, dy, 'move']);
    }],
    ['onPointerup', end], ['onPointercancel', end], ['onLostpointercapture', end],
  ];
}

function nativeWidgetNode(element, children, click, close, listeners, focusMarker, path) {
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
  if (isCheckedImageSurface(tag)) return checkedImageNode(properties, props, path);
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

function fileBackendTarget(url, post) {
  const pathAndQuery = url.slice('auto:back:'.length);
  const separator = pathAndQuery.indexOf('?');
  const path = separator === -1 ? pathAndQuery : pathAndQuery.slice(0, separator);
  if (!/^\/api(?:\/[a-zA-Z0-9_-]+)+$/.test(path) || post !== (separator === -1)) {
    throw new A2xTrap('Fail', 'http.client: invalid backend request');
  }
  if (post) return path;
  const json = pathAndQuery.slice(separator + 1);
  if (new TextEncoder().encode(json).length > _jsonMaxText) {
    throw new A2xTrap('Budget', 'http.client: JSON request exceeds the text limit');
  }
  let params;
  try {
    params = JSON.parse(json);
  } catch {
    throw new A2xTrap('Fail', 'http.client: invalid backend query');
  }
  if (params === null || typeof params !== 'object' || Array.isArray(params)) {
    throw new A2xTrap('Fail', 'http.client: invalid backend query');
  }
  const query = new URLSearchParams();
  for (const [name, value] of Object.entries(params)) {
    if (!['string', 'number', 'boolean'].includes(typeof value)) {
      throw new A2xTrap('Fail', 'http.client: invalid backend query parameter');
    }
    query.append(name, String(value));
  }
  return query.size === 0 ? path : `${path}?${query}`;
}

async function requestWidgetJson(request, signal, fileBackend) {
  const post = typeof request !== 'string';
  const url = post ? request.url : request;
  const scoped = url.startsWith('auto:back:');
  let target;
  if (scoped) {
    if (fileBackend === null) throw new A2xTrap('Fail', 'http.client: backend session unavailable');
    target = fileBackendTarget(url, post);
  } else {
    try {
      target = new URL(url, globalThis.location.href);
    } catch {
      throw new A2xTrap('Fail', 'http.client: invalid URL');
    }
    if (target.origin !== globalThis.location.origin || !['http:', 'https:'].includes(target.protocol)) {
      throw new A2xTrap('Fail', 'http.client: request must use the widget origin');
    }
  }
  const options = { signal, mode: 'same-origin', credentials: 'same-origin' };
  if (post) {
    if (new TextEncoder().encode(request.json).length > _jsonMaxText) {
      throw new A2xTrap('Budget', 'http.client: JSON request exceeds the text limit');
    }
    options.method = 'POST';
    options.headers = { 'Content-Type': 'application/json' };
    options.body = request.json;
  }
  try {
    const response = await (scoped ? fileBackend(target, options) : fetch(target, options));
    if (!response.ok) {
      await response.body?.cancel();
      throw new A2xTrap('Fail', 'http.client: response status ' + response.status);
    }
    const reader = response.body?.getReader();
    if (!reader) return new Uint8Array(0);
    const chunks = [];
    let length = 0;
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        length += value.length;
        if (length > _jsonMaxText) {
          await reader.cancel();
          throw new A2xTrap('Budget', 'http.client: JSON response exceeds the text limit');
        }
        chunks.push(value);
      }
    } finally {
      reader.releaseLock();
    }
    const bytes = new Uint8Array(length);
    let offset = 0;
    for (const chunk of chunks) {
      bytes.set(chunk, offset);
      offset += chunk.length;
    }
    return bytes;
  } catch (error) {
    if (signal.aborted || error instanceof A2xTrap) throw error;
    throw new A2xTrap('Fail', 'http.client: request failed');
  }
}

function createCheckedWidgetView(root, fileDirectory, fileMutation, preferences = [], fileOpen = null, fileBackend = null, shortcuts = {}) {
  const rendered = shallowRef(null);
  const pointerSamples = new WeakMap();
  const panSamples = new WeakMap();
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
  let initActionsScheduled = false;
  let initActionBudget = null;
  const pendingHttp = new Set();
  const hasHttp = Object.values(aceComponents).some(component =>
    Object.values(component.handlers).some(handler => handler.suspends));
  let nextTransition = 0;

  function draw(changes = new Map(), focus = []) {
    const frame = { models: new Map(), timerOwners: new Map(), initActions: new Map(), focus, focusTargets: new Map() };
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
        if (inputsChanged && Object.hasOwn(definition.handlers, 'Init')) {
          const handler = definition.handlers.Init;
          if (handler.types.length !== 0) throw new Error('ace.widget.init: Init must take no payload');
          if (handler.suspends) {
            frame.initActions.set(path, [{ path, handler, types: [], values: [] }]);
          } else {
            context.model = handler.run(...widgetArguments(context));
            collectWidgetFocus(context, context.model, frame.focus);
            const actions = Array.from(outboundWidgetActions(context, context.model));
            if (actions.length !== 0) frame.initActions.set(path, actions);
          }
        } else if (!inputsChanged && current.initActions.has(path)) {
          frame.initActions.set(path, current.initActions.get(path));
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
        // PLAN-118（T-06）：画布 arm——场景五平行列表（模型字段）→ 绘图 +
        // `onhit` 命中（与 AV/native 同一条 hit_test：rect 半宽高、circle 半径，
        // 逆序遍历）。命中 id 作为 `onhit` 的隐式实参派发。
        if (element.tag === 'canvas') {
          return checkedCanvasNode(element, owner, frame, path, dispatch);
        }
        let click = null;
        let close = null;
        const bindings = new Set();
        const listeners = new Map();
        for (const event of element.events) {
          const kind = event.kind.toLowerCase();
          if (bindings.has(kind)) throw new Error(`ace.widget.event: duplicate binding '${event.kind}'`);
          bindings.add(kind);
          // PLAN-118（T-06）：onhit 的命中 id 由渲染层注入（native 侧同款：
          // PackageEventInput::Hit），视图协议里不含显式实参——按 input 位校验。
          const acceptsInputValue = (element.tag === 'input' &&
            (kind === 'oninput' || kind === 'onchange' || kind === 'onenter') && event.types.length === 0) ||
            (kind === 'onhit' && event.types.length === 0);
          const pointerMove = element.tag === 'mouse-area' && kind === 'onmousemove' && event.types.length === 0;
          const action = checkedWidgetAction(owner, event, acceptsInputValue,
            pointerMove ? ['f32', 'f32'] : event.types);
          if (isCheckedImageSurface(element.tag) && (kind === 'onpan' || kind === 'onwheel')) {
            const enabled = () => active && current === frame && element.properties.get('disabled') !== 'true';
            const run = values => {
              if (enabled()) dispatch(frame, event.imageInput ? { ...action, values } : action);
            };
            if (kind === 'onpan') {
              for (const [property, listen] of checkedPanListeners(panSamples, enabled, run)) {
                listeners.set(property, listen);
              }
            } else {
              listeners.set('onWheel', domEvent => {
                if (!enabled() || !Number.isFinite(domEvent.deltaY)) return;
                domEvent.preventDefault();
                run([domEvent.deltaY]);
              });
            }
            continue;
          }
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
          registerFocusTarget(frame, owner, element.properties.get('id')), path);
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
    for (const transaction of pendingHttp) {
      for (const instance of transaction.models.values()) models.push([instance.model, instance.inputs]);
      models.push(Array.from(transaction.held));
    }
    a2x_widget_keep_models(models);
  }

  function publish(frame) {
    for (const transaction of pendingHttp) {
      if (!httpOwnersCurrent(transaction, frame)) cancelWidgetHttp(transaction);
    }
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
      current.initActions.clear();
      current.view = null;
    }
    savedPreferences = nextPreferences;
    current = frame;
    rendered.value = frame.view;
    if (frame.focus.length !== 0) void nextTick(() => applyFrameFocus(frame));
    if (mounted) {
      reconcileTimers(frame.timerOwners);
      startPendingMutation();
      scheduleInitActions();
    }
  }

  function scheduleInitActions() {
    if (current.initActions.size === 0) initActionBudget = null;
    if (current.initActions.size === 0 || initActionsScheduled || directoryRequest !== null) return;
    initActionsScheduled = true;
    // Mount and patch the VNodes before invoking any parent handlers.
    void nextTick(() => {
      initActionsScheduled = false;
      if (!mounted || current.initActions.size === 0 || directoryRequest !== null) return;
      const frame = current;
      const actions = Array.from(frame.initActions.values()).flat();
      frame.initActions.clear();
      initActionBudget ??= { remaining: 4096 };
      try {
        dispatchActions(frame, actions, null, initActionBudget);
      } catch (error) {
        initActionBudget = null;
        throw error;
      }
    });
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
          if (Array.from(pendingHttp).some(transaction => transaction.touched.has(path))) return;
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
      scheduleInitActions();
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

  function httpOwnersCurrent(transition, frame) {
    if (!active || frame === null) return false;
    for (const path of transition.touched) {
      const before = transition.models.get(path);
      const after = frame.models.get(path);
      if (!before || !after || after.definition !== before.definition || after.model !== before.model ||
          !sameWidgetValue(before.inputs, after.inputs)) return false;
    }
    return true;
  }

  function cancelWidgetHttp(transition) {
    pendingHttp.delete(transition);
    transition.controller?.abort();
    transition.iterator.return();
  }

  function* applyWidgetAction(transition, action) {
    if (transition.budget.remaining-- === 0) throw new Error('ace.widget.outbound: transition limit exceeded');
    transition.touched.add(action.path);
    if (transition.controller !== null && !httpOwnersCurrent(transition, current)) return;
    const instance = transition.models.get(action.path);
    let model = transition.changes.get(action.path) ?? instance.model;
    let inputs = instance.inputs;
    if (hasHttp) {
      if (transition.inputs.has(action.path)) {
        inputs = transition.inputs.get(action.path);
      } else {
        // Changes made before a yield belong to the transition's private state.
        [model, inputs] = a2x_widget_copy_state([model, inputs]);
        transition.inputs.set(action.path, inputs);
      }
    }
    const previous = withWidgetContext({ ...instance, inputs }, model);
    const payload = action.values.map((value, index) => widgetScalar(action.types[index], value));
    const result = action.handler.run(...widgetArguments({ ...instance, inputs, model: previous }), ...payload);
    model = action.handler.suspends ? yield* result : result;
    transition.changes.set(action.path, model);
    collectWidgetFocus(instance, model, transition.focus);
    for (const target of outboundWidgetActions(instance, model)) {
      yield* applyWidgetAction(transition, target);
    }
  }

  function* applyWidgetActions(transition, actions) {
    for (const action of actions) yield* applyWidgetAction(transition, action);
  }

  function dispatch(frame, action, enabled = null) {
    dispatchActions(frame, [action], enabled, { remaining: 4096 });
  }

  function dispatchActions(frame, actions, enabled, budget) {
    if (!active || current !== frame) return;
    a2x_widget_begin_frame();
    if (enabled !== null) {
      try {
        if (!enabled()) {
          keep(current);
          return;
        }
      } catch (error) {
        keep(current);
        throw error;
      }
    }
    const transition = {
      sequence: nextTransition++, models: frame.models, touched: new Set(), inputs: new Map(),
      changes: new Map(), focus: [], budget, held: new Set(), controller: null,
    };
    transition.iterator = applyWidgetActions(transition, actions);
    advanceWidgetActions(transition, 'next');
  }

  function advanceWidgetActions(transition, method, value) {
    try {
      const step = a2x_widget_step(transition.held, () => transition.iterator[method](value));
      if (!httpOwnersCurrent(transition, current)) {
        cancelWidgetHttp(transition);
        keep(current);
      } else if (step.done) {
        pendingHttp.delete(transition);
        commitWidgetChanges(current, transition.changes, transition.focus);
      } else {
        parkWidgetHttp(transition, step.value);
      }
    } catch (error) {
      cancelWidgetHttp(transition);
      keep(current);
      throw error;
    }
  }

  function parkWidgetHttp(transition, request) {
    if (typeof request !== 'string' && (typeof request?.url !== 'string' || typeof request?.json !== 'string')) {
      throw new Error('ace.widget.http: invalid request');
    }
    if (transition.controller === null) {
      transition.models = new Map(transition.models);
      transition.controller = new AbortController();
    }
    const overlapping = Array.from(pendingHttp).filter(other => other !== transition &&
      Array.from(transition.touched).some(path => other.touched.has(path)));
    if (overlapping.some(other => other.sequence > transition.sequence)) {
      cancelWidgetHttp(transition);
      keep(current);
      return;
    }
    for (const other of overlapping) cancelWidgetHttp(other);
    transition.workBudget = a2x_widget_work_budget();
    pendingHttp.add(transition);
    keep(current);
    void resumeWidgetHttp(transition, request);
  }

  async function resumeWidgetHttp(transition, request) {
    let method = 'next';
    let value;
    try {
      value = await requestWidgetJson(request, transition.controller.signal, fileBackend);
    } catch (error) {
      method = 'throw';
      value = error;
    }
    if (!pendingHttp.has(transition)) return;
    a2x_widget_begin_frame(transition.workBudget);
    advanceWidgetActions(transition, method, value);
  }

  function commitWidgetChanges(frame, changes, focus) {
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
  }

  function onShortcut(event) {
    if (event.defaultPrevented || event.repeat || event.altKey || event.ctrlKey || event.metaKey ||
        !Object.hasOwn(shortcuts, event.key) || current === null) return;
    if (event.key !== 'Escape' && event.target instanceof Element &&
        event.target.closest('input, textarea, select, [contenteditable="true"]')) return;
    const owner = current.models.get('root');
    const action = checkedWidgetAction(owner, { message: shortcuts[event.key], types: [], values: [] });
    event.preventDefault();
    dispatch(current, action);
  }
  const hasShortcuts = Object.keys(shortcuts).length !== 0;

  onMounted(() => {
    if (current === null) return;
    mounted = true;
    if (hasShortcuts) window.addEventListener('keydown', onShortcut);
    reconcileTimers(current.timerOwners);
    if (fileDirectory !== null) {
      const model = current.models.get('root').model;
      const path = pendingDirectoryPath('root', model);
      if (path !== null) startDirectoryRequest(current, new Map([['root', model]]), path);
    }
    startPendingMutation();
    scheduleInitActions();
  });

  onBeforeUnmount(() => {
    if (hasShortcuts) window.removeEventListener('keydown', onShortcut);
    active = false;
    mounted = false;
    for (const transition of pendingHttp) cancelWidgetHttp(transition);
    cancelDirectoryRequest();
    mutationRequest?.controller.abort();
    mutationRequest = null;
    reconcileTimers(new Map());
    current?.models.clear();
    current?.timerOwners.clear();
    current?.initActions.clear();
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
