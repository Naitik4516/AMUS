<script>
    import {
        House,
        Bookmark,
        ChartNoAxesColumn,
        Settings,
    } from "@lucide/svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/state";

    const menuItems = [
        { label: "Home", Icon: House, route: "/" },
        { label: "Library", Icon: Bookmark, route: "/library" },
        { label: "Stats", Icon: ChartNoAxesColumn, route: "/library/stats" },
        { label: "Settings", Icon: Settings, route: "/settings" },
    ];

    const currentRoute = $derived(page.url.pathname);
</script>

<div
    class="fixed left-4 top-[30%] flex flex-col gap-3 p-6 rounded-full bg-gray-300/5 border-zinc-700/30 border text-white shadow-lg z-40 w-21"
>
    {#each menuItems as item (item)}
        {@const active = currentRoute === item.route}
        <button
            onclick={() => goto(item.route)}
            class="group flex items-center w-11 h-11 -ml-1 hover:w-28
                          transition-all duration-400 ease-out overflow-hidden
                          rounded-full bg-white/5 hover:bg-gray-300/10
                          ring-1 ring-gray-400/20 hover:ring-gray-400/40
                          {active
                ? 'text-accent'
                : 'text-slate-200 hover:text-white'}"
            aria-label={item.label}
        >
            <div class="flex w-11 justify-center items-center shrink-0">
                <item.Icon size={24} strokeWidth={2.5} />
            </div>
            <span
                class="pr-4 pl-0 opacity-0 translate-x-2
                          group-hover:opacity-100 group-hover:translate-x-0
                          group-focus:opacity-100 group-focus:translate-x-0
                          transition-all duration-300 whitespace-nowrap text-sm font-medium tracking-wide"
            >
                {item.label}
            </span>
        </button>
    {/each}
</div>
