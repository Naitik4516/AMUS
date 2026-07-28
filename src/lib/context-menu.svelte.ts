interface ContextMenuState {
  component: any;
  props: Record<string, unknown>;
}

export const contextMenu = $state<{ current: ContextMenuState | null }>({ current: null });

export function openContextMenu(component: any, props: Record<string, unknown>) {
  contextMenu.current = { component, props };
}

export function closeContextMenu() {
  contextMenu.current = null;
}
