<script>
    import {
        House,
        Bookmark,
        ChartNoAxesColumn,
        Settings,
    } from "@lucide/svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import { flip } from "svelte/animate";

    const menuItems = [
        { label: "Home", Icon: House, route: "/" },
        { label: "Library", Icon: Bookmark, route: "/library" },
        { label: "Stats", Icon: ChartNoAxesColumn, route: "/library/stats" },
        { label: "Settings", Icon: Settings, route: "/settings" },
    ];

    const currentRoute = $derived(page.url.pathname);
</script>

<div
    class="fixed h-full left-4 flex flex-col items-center align-middle justify-between py-4"
>
    <div
        class="flex flex-col gap-3 p-6 rounded-full bg-zinc-900/20 border-zinc-700/30 border-2 text-white shadow-lg my-auto -translate-y-20 z-10 w-21"
    >
        {#each menuItems as item (item)}
            <button
                animate:flip
                onclick={() => goto(item.route)}
                class={`group flex items-center w-11 h-11 -ml-1 z-50 hover:w-32 focus:w-48
                          transition-all duration-400 ease-out overflow-hidden
                          rounded-full bg-zinc-800/10 hover:bg-zinc-800/50 focus:bg-white/10
                          backdrop-blur-md ring-2 ring-zinc-600/20 hover:ring-zinc-600/30 focus:ring-zinc-600/40
                          focus:outline-none
                          ${
                              currentRoute === item.route
                                  ? "text-accent"
                                  : "text-slate-200 hover:text-white"
                          }`}
                aria-label={item.label}
            >
                <div class="flex w-11 justify-center items-center shrink-0">
                    <item.Icon size={24} class="drop-shadow" />
                </div>
                <span
                    class="pr-4 pl-0 opacity-0 translate-x-2
                          group-hover:opacity-100 group-hover:translate-x-0
                          group-focus:opacity-100 group-focus:translate-x-0
                          transition-all duration-300 whitespace-nowrap text-sm font-medium tracking-wide z-50"
                >
                    {item.label}
                </span>
            </button>
        {/each}
    </div>
</div>
