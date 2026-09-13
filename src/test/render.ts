import { createRenderer, h, type Component } from "vue";

// 用 Vue 自带的渲染器运行真实组件与事件，避免为这些交互用例引入 DOM 依赖。
export interface TestNode {
  tag: string;
  tagName: string;
  text: string;
  parent: TestNode | null;
  children: TestNode[];
  props: Record<string, unknown>;
  value: unknown;
  type: string;
  multiple: boolean;
  selected: boolean;
  readonly options: TestNode[];
  readonly textContent: string;
  getRootNode: () => TestNode;
  addEventListener: () => void;
  removeEventListener: () => void;
  focus: () => void;
}

function node(tag: string, text = ""): TestNode {
  return {
    tag,
    tagName: tag.toUpperCase(),
    text,
    parent: null,
    children: [],
    props: {},
    value: "",
    type: "",
    multiple: false,
    selected: false,
    get options() {
      return this.children.filter((child) => child.tag === "option");
    },
    get textContent() {
      if (this.tag === "#comment") return "";
      return this.text + this.children.map((child) => child.textContent).join("");
    },
    getRootNode() {
      return this.parent?.getRootNode() ?? this;
    },
    addEventListener() {},
    removeEventListener() {},
    focus() {},
  };
}

function remove(child: TestNode) {
  if (child.parent) {
    const index = child.parent.children.indexOf(child);
    if (index >= 0) child.parent.children.splice(index, 1);
  }
  child.parent = null;
}

function insert(child: TestNode, parent: TestNode, anchor: TestNode | null = null) {
  remove(child);
  const index = anchor ? parent.children.indexOf(anchor) : -1;
  parent.children.splice(index < 0 ? parent.children.length : index, 0, child);
  child.parent = parent;
}

const renderer = createRenderer<TestNode, TestNode>({
  createElement: (tag) => node(tag),
  createText: (text) => node("#text", text),
  createComment: (text) => node("#comment", text),
  setText: (target, text) => { target.text = text; },
  setElementText: (target, text) => {
    target.children = [];
    target.text = text;
  },
  parentNode: (target) => target.parent,
  nextSibling: (target) => {
    const siblings = target.parent?.children ?? [];
    return siblings[siblings.indexOf(target) + 1] ?? null;
  },
  insert,
  remove,
  patchProp: (target, key, _previous, value) => {
    target.props[key] = value;
    if (key === "value") target.value = value;
    if (key === "type") target.type = String(value);
    if (key === "multiple") target.multiple = Boolean(value);
    if (key === "selected") target.selected = Boolean(value);
  },
  setScopeId() {},
  insertStaticContent: (content, parent, anchor) => {
    const target = node("#static", content);
    insert(target, parent, anchor);
    return [target, target];
  },
});

export function mount(component: Component, props: Record<string, unknown> = {}) {
  const root = node("root");
  const app = renderer.createApp({ render: () => h(component, props) });
  app.mount(root);
  return { root, unmount: () => app.unmount() };
}

export function findAll(root: TestNode, predicate: (target: TestNode) => boolean): TestNode[] {
  return [
    ...(predicate(root) ? [root] : []),
    ...root.children.flatMap((child) => findAll(child, predicate)),
  ];
}

export function find(root: TestNode, predicate: (target: TestNode) => boolean): TestNode {
  const target = findAll(root, predicate)[0];
  if (!target) throw new Error("未找到预期组件节点");
  return target;
}

export function hasClass(target: TestNode, className: string): boolean {
  return String(target.props.class ?? "").split(/\s+/).includes(className);
}

export function trigger(target: TestNode, handler: string, ...args: unknown[]) {
  const callback = target.props[handler];
  if (typeof callback !== "function") throw new Error(`节点缺少事件: ${handler}`);
  return callback(...(args.length ? args : [{ preventDefault() {}, stopPropagation() {} }]));
}
