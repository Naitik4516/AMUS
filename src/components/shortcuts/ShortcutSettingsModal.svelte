<script lang="ts">
    import * as Dialog from "$components/ui/dialog/index.js";
    import * as Accordion from "$components/ui/accordion/index.js";
    import * as Kbd from "$components/ui/kbd/index.js";
    import Button from "$components/ui/button/button.svelte";
    import {
        ALL_SHORTCUTS,
        GLOBAL_SHORTCUT_ACTIONS,
        type ShortcutAction,
        type ShortcutCategory,
        type ShortcutBinding,
        type GlobalShortcutAction,
        getEffectiveBindings,
        getEffectiveGlobalBinding,
        customBindings,
        globalCustomBindings,
        globalShortcutFlags,
        setGlobalShortcutEnabled,
        addBinding,
        removeSingleBinding,
        setCustomBindings,
        resetGlobalCustomBinding,
        setGlobalCustomBinding,
        resetAllBindings,
        formatBinding,
        bindingsEqual,
        bindingFromEvent,
        findConflicts,
        removeBindingFromAllOthers,
    } from "$lib/shortcuts.svelte";
    import ToggleSwitch from "$components/ui/ToggleSwitch.svelte";
    import X from "@lucide/svelte/icons/x";
    import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
    import AlertTriangle from "@lucide/svelte/icons/alert-triangle";

    let {
        open = $bindable(false),
    }: {
        open?: boolean;
    } = $props();

    const CATEGORIES: { key: ShortcutCategory; label: string }[] = [
        { key: "playback", label: "Playback" },
        { key: "seeking", label: "Seeking" },
        { key: "volume", label: "Volume" },
        { key: "library", label: "Library / Search" },
        { key: "queue", label: "Queue" },
        { key: "navigation", label: "Navigation" },
        { key: "window", label: "Window" },
    ];

    let dialogEl = $state<HTMLDialogElement>();
    let editingId = $state<string | null>(null);
    let capturing = $state(false);
    let captureBuffer = $state<string>("");
    let conflictWarning = $state<string | null>(null);
    let pendingBindings = $state<ShortcutBinding[]>([]);
    let activeTab = $state<"local" | "global">("local");

    $effect(() => {
        if (!dialogEl) return;
        if (open) {
            if (!dialogEl.open) {
                dialogEl.showModal();
            }
        } else {
            if (dialogEl.open) {
                dialogEl.close();
            }
        }
    });

    $effect(() => {
        if (!capturing) return;

        const handleGlobalCapture = (e: KeyboardEvent) => {
            e.preventDefault();
            e.stopPropagation();
            e.stopImmediatePropagation();
            handleCaptureKey(e);
        };

        window.addEventListener("keydown", handleGlobalCapture, true);
        return () => {
            window.removeEventListener("keydown", handleGlobalCapture, true);
        };
    });

    function startEdit(action: ShortcutAction | GlobalShortcutAction) {
        if (capturing) return;
        editingId = action.id;
        capturing = true;
        captureBuffer = "Press a key...";
        conflictWarning = null;
        pendingBindings = [];
    }

    function getCaptureModifierString(e: KeyboardEvent): string | null {
        const isMac =
            typeof navigator !== "undefined" &&
            navigator.userAgent.includes("Mac");
        const modParts: string[] = [];
        if (isMac) {
            if (e.ctrlKey || e.metaKey) modParts.push("Cmd");
            if (e.shiftKey) modParts.push("Shift");
            if (e.altKey) modParts.push("Option");
        } else {
            if (e.ctrlKey) modParts.push("Ctrl");
            if (e.shiftKey) modParts.push("Shift");
            if (e.altKey) modParts.push("Alt");
            if (e.metaKey) modParts.push("Win");
        }
        return modParts.length > 0 ? modParts.join(" + ") + " + \u2026" : null;
    }

    function handleCaptureKey(e: KeyboardEvent) {
        if (!capturing) return;

        if (e.key === "Escape") {
            cancelCapture();
            return;
        }

        if (["Control", "Shift", "Alt", "Meta", "Process"].includes(e.key)) {
            const modStr = getCaptureModifierString(e);
            captureBuffer = modStr ?? "Press a key\u2026";
            return;
        }

        const binding = bindingFromEvent(e);
        if (!binding) return;

        if (editingId?.startsWith("global_")) {
            const conflicts = findConflicts(editingId, [binding]);
            if (conflicts.length > 0) {
                conflictWarning = `Conflicts with: ${conflicts.join(", ")}`;
                pendingBindings = [binding];
                capturing = false;
                captureBuffer = formatBinding(binding).join(" + ");
            } else {
                setGlobalCustomBinding(editingId, binding);
                cancelCapture();
            }
            return;
        }

        const action = ALL_SHORTCUTS.find((a) => a.id === editingId);
        if (!action) return;

        const existing = getEffectiveBindings(action);
        const alreadyAssigned = existing.some((b) => bindingsEqual(b, binding));
        if (alreadyAssigned) {
            conflictWarning = "Already assigned";
            pendingBindings = [binding];
            capturing = false;
            captureBuffer = formatBinding(binding).join(" + ");
            return;
        }

        const newBindings = [binding];
        const conflicts = findConflicts(editingId!, newBindings);

        if (conflicts.length > 0) {
            conflictWarning = `Conflicts with: ${conflicts.join(", ")}`;
            pendingBindings = newBindings;
            capturing = false;
            captureBuffer = formatBinding(binding).join(" + ");
        } else {
            addBinding(editingId!, binding);
            cancelCapture();
        }
    }

    async function confirmConflictOverwrite() {
        if (editingId && pendingBindings.length > 0) {
            const binding = pendingBindings[0];
            await removeBindingFromAllOthers(editingId, binding);

            if (editingId.startsWith("global_")) {
                await setGlobalCustomBinding(editingId, binding);
            } else {
                await addBinding(editingId, binding);
            }
            cancelCapture();
        }
    }

    function cancelCapture() {
        editingId = null;
        capturing = false;
        captureBuffer = "";
        conflictWarning = null;
        pendingBindings = [];
    }

    async function handleResetAll() {
        await resetAllBindings();
        cancelCapture();
    }

    function isEditing(id: string) {
        return editingId === id && capturing;
    }

    function showConflict(id: string) {
        return editingId === id && conflictWarning !== null;
    }
</script>

<dialog
    bind:this={dialogEl}
    id="shortcut-settings-modal"
    onclose={() => {
        open = false;
        cancelCapture();
    }}
    class="w-4xl max-h-[80vh] flex flex-col relative p-5 rounded-3xl bg-card/70 backdrop-blur-2xl border m-auto text-white"
>
    <div>
        <div class="text-2xl font-black mb-2">Keyboard Shortcuts</div>
        <div>Customize keyboard shortcuts for all actions.</div>
    </div>

    <div class="absolute top-4 right-4">
        <Button
            variant="ghost"
            size="icon"
            title="Close"
            command="close"
            commandfor="shortcut-settings-modal"
        >
            <X size={16} />
        </Button>
    </div>

    <div class="flex gap-2 my-4 border-b border-white/10 pb-3">
        <button
            class="px-4 py-2 rounded-full text-sm font-medium transition-colors cursor-pointer {activeTab ===
            'local'
                ? 'bg-accent text-accent-foreground'
                : 'hover:bg-white/10'}"
            onclick={() => (activeTab = "local")}>App Shortcuts</button
        >
        <button
            class="px-4 py-2 rounded-full text-sm font-medium transition-colors cursor-pointer {activeTab ===
            'global'
                ? 'bg-accent text-accent-foreground'
                : 'hover:bg-white/10'}"
            onclick={() => (activeTab = "global")}>Global Shortcuts</button
        >
    </div>

    {#if activeTab === "local"}
        <Accordion.Root type="multiple" class="space-y-2 overflow-y-auto">
            {#each CATEGORIES as cat}
                {@const actions = ALL_SHORTCUTS.filter(
                    (a) => a.category === cat.key,
                )}
                {#if actions.length > 0}
                    <Accordion.Item value={cat.key}>
                        <Accordion.Trigger>
                            <span class="font-bold text-white">{cat.label}</span
                            >
                        </Accordion.Trigger>
                        <Accordion.Content>
                            <div class="divide-y divide-white/5">
                                {#each actions as action}
                                    <div
                                        class="flex items-center justify-between py-1.5 gap-3"
                                    >
                                        <span
                                            class="text-sm font-medium text-white truncate min-w-0"
                                        >
                                            {action.label}
                                        </span>

                                        <div
                                            class="flex items-center gap-1.5 shrink-0 flex-wrap justify-end"
                                        >
                                            {#each getEffectiveBindings(action) as binding}
                                                {@const parts = formatBinding(binding)}
                                                <div
                                                    class="flex items-center gap-1.5 border border-white/10 rounded-lg px-2 py-1"
                                                >
                                                    <Kbd.Group>
                                                        {#each parts as keyPart, i (i)}
                                                            <Kbd.Root
                                                                >{keyPart}</Kbd.Root
                                                            >
                                                            {#if i < parts.length - 1}
                                                                <span
                                                                    class="text-gray-400"
                                                                    >+</span
                                                                >
                                                            {/if}
                                                        {/each}
                                                    </Kbd.Group>
                                                    <button
                                                        class="p-0.5 rounded hover:bg-white/10 transition-colors text-gray-500 hover:text-red-400 cursor-pointer"
                                                        onclick={() =>
                                                            removeSingleBinding(
                                                                action.id,
                                                                binding,
                                                            )}
                                                        title="Remove this shortcut"
                                                    >
                                                        <X size={10} />
                                                    </button>
                                                </div>
                                            {/each}

                                            {#if isEditing(action.id)}
                                                {#if showConflict(action.id)}
                                                    <span
                                                        class="text-red-400 text-xs"
                                                        >{conflictWarning}</span
                                                    >
                                                    <Button
                                                        size="xs"
                                                        variant="default"
                                                        onclick={confirmConflictOverwrite}
                                                    >
                                                        Overwrite
                                                    </Button>
                                                    <Button
                                                        size="xs"
                                                        variant="ghost"
                                                        onclick={cancelCapture}
                                                    >
                                                        Cancel
                                                    </Button>
                                                {:else}
                                                    <Kbd.Root class="text-accent"
                                                        >{captureBuffer}</Kbd.Root
                                                    >
                                                    <Button
                                                        size="xs"
                                                        variant="ghost"
                                                        onclick={cancelCapture}
                                                    >
                                                        <X size={14} /> Cancel
                                                    </Button>
                                                {/if}
                                            {:else}
                                                <button
                                                    class="px-2 py-0.5 rounded text-xs font-medium text-gray-400 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
                                                    onclick={() =>
                                                        startEdit(action)}
                                                    title="Add a shortcut"
                                                    >+ Add</button
                                                >
                                            {/if}
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        </Accordion.Content>
                    </Accordion.Item>
                {/if}
            {/each}
        </Accordion.Root>

        <div
            class="flex pt-4 border-t border-white/10 mt-4"
        >
            <Button variant="outline" size="sm" onclick={handleResetAll}>
                <RotateCcw size={14} /> Reset All
            </Button>
        </div>
    {:else}
        <div class="space-y-1">
            <p class="text-xs text-muted-foreground italic mb-4">
                Global shortcuts work even when app is minimized.
            </p>
            {#each GLOBAL_SHORTCUT_ACTIONS as g}
                {@const binding = getEffectiveGlobalBinding(g)}
                <div
                    class="flex items-center justify-between px-3 py-2 rounded-xl hover:bg-white/5 transition-colors gap-3"
                >
                    <div class="flex items-center gap-3 truncate min-w-0">
                        <ToggleSwitch
                            checked={globalShortcutFlags[g.id]}
                            onchange={(checked) =>
                                setGlobalShortcutEnabled(g.id, checked)}
                            disabled={g.defaultBinding === null &&
                                !globalCustomBindings[g.id]}
                        />
                        <span class="text-sm font-medium text-white truncate"
                            >{g.label}</span
                        >
                    </div>

                    <div class="flex items-center gap-1.5 shrink-0">
                        {#if binding}
                            {@const parts = formatBinding(binding)}
                            <div
                                class="flex items-center gap-1 bg-white/5 border border-white/10 rounded-lg px-2 py-0.5"
                            >
                                <Kbd.Group>
                                    {#each parts as keyPart, i (i)}
                                        <Kbd.Root>{keyPart}</Kbd.Root>
                                        {#if i < parts.length - 1}
                                            <span class="text-gray-400">+</span>
                                        {/if}
                                    {/each}
                                </Kbd.Group>
                                {#if globalCustomBindings[g.id]}
                                    <button
                                        class="p-0.5 rounded hover:bg-white/10 transition-colors text-gray-500 hover:text-red-400 cursor-pointer"
                                        onclick={() => {
                                            resetGlobalCustomBinding(g.id);
                                        }}
                                        title="Remove this shortcut"
                                    >
                                        <X size={10} />
                                    </button>
                                {/if}
                            </div>
                        {:else}
                            <span class="text-xs text-gray-600 italic"
                                >No binding</span
                            >
                        {/if}

                        {#if isEditing(g.id)}
                            {#if showConflict(g.id)}
                                <span class="text-red-400 text-xs"
                                    >{conflictWarning}</span
                                >
                                <Button
                                    size="xs"
                                    variant="default"
                                    onclick={confirmConflictOverwrite}
                                >
                                    Overwrite
                                </Button>
                                <Button
                                    size="xs"
                                    variant="ghost"
                                    onclick={cancelCapture}
                                >
                                    Cancel
                                </Button>
                            {:else}
                                <Kbd.Root class="text-accent"
                                    >{captureBuffer}</Kbd.Root
                                >
                                <Button
                                    size="xs"
                                    variant="ghost"
                                    onclick={cancelCapture}
                                >
                                    <X size={14} /> Cancel
                                </Button>
                            {/if}
                        {:else}
                            <button
                                class="px-2 py-0.5 rounded text-xs font-medium text-gray-400 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
                                onclick={() => startEdit(g)}
                                title="Add a shortcut">+ Add</button
                            >
                        {/if}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</dialog>
