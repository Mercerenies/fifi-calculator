
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
}

interface EventFunctions {

}
