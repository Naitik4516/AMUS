interface ContextMenuState {
  component: any;
  props: Record<string, unknown>;
}

export interface ConfirmDialogState {
  open: boolean;
  title: string;
  message: string;
  confirmLabel: string;
  onConfirm: () => void | Promise<void>;
}

export const contextMenu = $state<{ current: ContextMenuState | null }>({ current: null });
export const confirmDialog = $state<ConfirmDialogState>({
  open: false,
  title: "",
  message: "",
  confirmLabel: "Delete",
  onConfirm: () => {},
});

export function openContextMenu(component: any, props: Record<string, unknown>) {
  contextMenu.current = { component, props };
}

export function closeContextMenu() {
  contextMenu.current = null;
}

export function openConfirmDialog(state: Omit<ConfirmDialogState, "open">) {
  confirmDialog.open = true;
  confirmDialog.title = state.title;
  confirmDialog.message = state.message;
  confirmDialog.confirmLabel = state.confirmLabel;
  confirmDialog.onConfirm = state.onConfirm;
}

export function closeConfirmDialog() {
  confirmDialog.open = false;
}
