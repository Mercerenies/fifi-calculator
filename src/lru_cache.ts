
export class LruCache<K, V> {
  readonly maxCacheSize: number;

  private mapping: Map<K, DataNode<readonly [K, V]>> = new Map();
  private headNode: HeadNode<readonly [K, V]> = headNode();

  constructor(maxCacheSize: number) {
    if (maxCacheSize < 1) {
      throw new Error('LruCache maxCacheSize must be > 0');
    }
    this.maxCacheSize = maxCacheSize;
  }

  get(key: K): V | undefined {
    const node = this.mapping.get(key);
    if (node === undefined) {
      return undefined;
    }
    this.moveToHead(node);
    return node.data[1];
  }

  set(key: K, value: V): void {
    if (this.mapping.size >= this.maxCacheSize) {
      this.removeLeastRecentlyUsed();
    }
    const node = dataNode([key, value] as const);
    this.mapping.set(key, node);
    this.moveToHead(node);
  }

  private moveToHead(node: DataNode<readonly [K, V]>): void {
    remove(node);
    insert(this.headNode, node, this.headNode.next);
  }

  private removeLeastRecentlyUsed() {
    const node = this.headNode.prev;
    if (!isHeadNode(node)) {
      remove(node);
      this.mapping.delete(node.data[0]);
    }
  }
}

interface DataNode<T> {
  prev: Node<T>;
  next: Node<T>;
  data: T;
}

interface HeadNode<T> {
  prev: Node<T>;
  next: Node<T>;
  __isHead: true;
}

type Node<T> = DataNode<T> | HeadNode<T>;

function isHeadNode<T>(node: Node<T>): node is HeadNode<T> {
  return (node as any).__isHead;
}

function insert<T>(prev: Node<T>, elem: Node<T>, next: Node<T>): void {
  elem.prev = prev;
  elem.next = next;
  prev.next = elem;
  next.prev = elem;
}

function remove<T>(elem: Node<T>): void {
  elem.prev.next = elem.next;
  elem.next.prev = elem.prev;
  elem.prev = elem;
  elem.next = elem;
}

function headNode<T>(): HeadNode<T> {
  const headNode: any = { __isHead: true };
  headNode.prev = headNode;
  headNode.next = headNode;
  return headNode;
}

function dataNode<T>(data: T): DataNode<T> {
  const node: any = { data };
  node.prev = node;
  node.next = node;
  return node;
}
