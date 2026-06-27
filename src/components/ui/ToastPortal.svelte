<script lang="ts">
    import { toast, type ToastType } from "$lib/stores/toast.svelte";
    import Icon from "./Icon.svelte";

    const icons: Record<ToastType, string> = {
        success: "check-circle",
        error: "alert-circle",
        info: "info",
        warning: "alert-triangle",
    };

    const colors: Record<ToastType, string> = {
        success: "bg-emerald-500/90 border-emerald-400/30",
        error: "bg-rose-500/90 border-rose-400/30",
        info: "bg-sky-500/90 border-sky-400/30",
        warning: "bg-amber-500/90 border-amber-400/30",
    };
</script>

{#if toast.toasts.length > 0}
    <div
        class="fixed top-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none"
        role="region"
        aria-live="polite"
        aria-label="Notifications"
    >
        {#each toast.toasts as t (t.id)}
            <div
                class="pointer-events-auto flex items-start gap-3 rounded-lg border px-4 py-3 min-w-[280px] max-w-[400px] shadow-xl backdrop-blur-md animate-slide-in {colors[t.type]}"
                role="alert"
            >
                <Icon name={icons[t.type]} size={20} class="flex-shrink-0 mt-0.5 text-white/90" />
                <p class="flex-1 text-sm text-white">{t.message}</p>
                <button
                    type="button"
                    class="flex-shrink-0 text-white/60 hover:text-white transition-colors"
                    onclick={() => toast.dismiss(t.id)}
                    aria-label="Dismiss"
                >
                    <Icon name="x" size={16} />
                </button>
            </div>
        {/each}
    </div>
{/if}

<style>
    @keyframes slide-in {
        from {
            opacity: 0;
            transform: translateX(100%);
        }
        to {
            opacity: 1;
            transform: translateX(0);
        }
    }
    .animate-slide-in {
        animation: slide-in 0.3s ease-out;
    }
    @media (prefers-reduced-motion: reduce) {
        .animate-slide-in {
            animation: none;
        }
    }
</style>