export type ToastType = "success" | "error" | "info" | "warning";

interface Toast {
    id: string;
    message: string;
    type: ToastType;
    duration?: number;
}

class ToastStore {
    toasts = $state<Toast[]>([]);

    show(message: string, type: Toast["type"] = "info", duration = 3000) {
        const id = crypto.randomUUID();
        this.toasts = [...this.toasts, { id, message, type, duration }];
        if (duration > 0) setTimeout(() => this.dismiss(id), duration);
    }

    dismiss(id: string) {
        this.toasts = this.toasts.filter((t) => t.id !== id);
    }

    success(message: string, duration = 3000) {
        this.show(message, "success", duration);
    }

    error(message: string, duration = 5000) {
        this.show(message, "error", duration);
    }

    info(message: string, duration = 3000) {
        this.show(message, "info", duration);
    }

    warning(message: string, duration = 4000) {
        this.show(message, "warning", duration);
    }
}

export const toast = new ToastStore();