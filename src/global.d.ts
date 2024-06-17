
interface CommandOptions {
  argument: number | null,
  keepModifier: boolean,
}

interface OsFunctions {
  type(): Promise<OsType>;
}

interface EventFunctions {
  listen: EventListen;
}

interface EventListen {
  (event: 'refresh-stack', callback: EventCallback<RefreshStackPayload>): Promise<UnlistenFunction>;
  (event: 'refresh-undo-availability', callback: EventCallback<UndoAvailabilityPayload>): Promise<UnlistenFunction>;
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

interface UndoAvailabilityPayload {
  hasUndos: boolean;
  hasRedos: boolean;
}

interface ShowErrorPayload {
  errorMessage: string;
}

type OsType = "Linux" | "Darwin" | "Windows_NT";
