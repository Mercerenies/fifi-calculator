
interface Window {
  __TAURI__: {
    tauri: TauriFunctions,
    event: EventFunctions,
    os: OsFunctions,
  };
}

interface TauriFunctions {
  invoke: TauriInvoke,
}

interface TauriInvoke {
  (command: 'submit_number', args: { value: string }): Promise<void>;
  (command: 'math_command', args: { commandName: string, prefixArgument: number | null }): Promise<void>;
}

interface OsFunctions {
  type(): Promise<OsType>;
}

interface EventFunctions {
  listen: EventListen;
}

interface EventListen {
  (event: 'refresh-stack', callback: EventCallback<RefreshStackPayload>): Promise<UnlistenFunction>;
  (event: 'show-error', callback: EventCallback<ShowErrorPayload>): Promise<UnlistenFunction>;
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

interface ShowErrorPayload {
  errorMessage: string;
}

type OsType = "Linux" | "Darwin" | "Windows_NT";
