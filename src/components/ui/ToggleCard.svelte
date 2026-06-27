<script lang="ts">
    import ToggleSwitch from "$components/ui/ToggleSwitch.svelte";

    let {
        title,
        description,
        checked = $bindable(false),
        onchange,
        icon: Icon,
        iconActiveClass = "",
    }: {
        title: string;
        description: string;
        checked?: boolean;
        onchange?: (checked: boolean) => void;
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        icon?: any;
        iconActiveClass?: string;
    } = $props();
</script>

<div
    class="bg-card/50 backdrop-blur-lg border border-border/60 rounded-3xl shadow-lg p-6"
>
    <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-3">
            {#if Icon}
                <Icon
                    size={20}
                    class={checked ? iconActiveClass : "text-gray-500"}
                />
            {/if}
            <h3 class="font-bold text-white text-lg">{title}</h3>
        </div>
        <ToggleSwitch {checked} {onchange} />
    </div>
    <p class="text-sm text-gray-400">{description}</p>
</div>
