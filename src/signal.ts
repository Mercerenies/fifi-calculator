
// Light-weight signalling and event-handling system.

export class Signal<A> {
  private listeners: SignalListener<A>[] = [];

  addListener(listener: SignalListener<A>): void {
    if (!this.listeners.includes(listener)) {
      this.listeners.push(listener);
    }
  }

  removeListener(listener: SignalListener<A>): void {
    const index = this.listeners.indexOf(listener);
    if (index !== -1) {
      this.listeners.splice(index, 1);
    }
  }

  emit(arg: A): void {
    // Just in case a listener removes itself while iterating, make a
    // defensive copy of the array here.
    const listeners = this.listeners.slice();
    for (const listener of listeners) {
      listener(arg, this);
    }
  }
}

export type SignalListener<A> = (arg: A, signal: Signal<A>) => void;

// Signal listener that removes itself after firing once.
export function oneShot<A>(listener: SignalListener<A>): SignalListener<A> {
  const oneShotListener = (arg: A, signal: Signal<A>) => {
    listener(arg, signal);
    signal.removeListener(oneShotListener);
  };
  return oneShotListener;
}
