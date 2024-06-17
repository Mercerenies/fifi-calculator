
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
