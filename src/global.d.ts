
interface Window {
  __TAURI__: {
    tauri: TauriFunctions,
    event: EventFunctions,
  };
}

interface TauriFunctions {
  invoke: TauriInvoke,
}

interface TauriInvoke {
  (command: 'submit_integer', args: { value: number }): Promise<void>;
  (command: 'math_command', args: { command_name: string }): Promise<void>;
}

interface EventFunctions {
  listen: EventListen;
}

interface EventListen {
  (event: 'refresh-stack', callback: EventCallback<RefreshStackPayload>): Promise<UnlistenFunction>;
}

type EventCallback<T> = (event: EventBody<T>) => void;

interface EventBody<T> {
  event: string;
  id: number;
  payload: T;
  windowLabel: string;
}

type UnlistenFunction = () => void;

interface RefreshStackPayload {
  stack: string[];
}
