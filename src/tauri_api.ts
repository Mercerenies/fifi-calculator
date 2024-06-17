
const __TAURI__ = window.__TAURI__;

class TauriApi {
  osType(): Promise<OsType> {
    return __TAURI__.os.type()
  }

  listen(event: 'refresh-stack', callback: EventCallback<RefreshStackPayload>): Promise<UnlistenFunction>;
  listen(event: 'refresh-undo-availability', callback: EventCallback<UndoAvailabilityPayload>): Promise<UnlistenFunction>;
  listen(event: 'show-error', callback: EventCallback<ShowErrorPayload>): Promise<UnlistenFunction>;
  listen(event: any, callback: EventCallback<any>): Promise<UnlistenFunction> {
    return __TAURI__.event.listen(event, callback);
  }
}

export const TAURI = new TauriApi();
