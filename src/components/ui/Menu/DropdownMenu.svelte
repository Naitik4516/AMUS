<script lang="ts">
    import { slide } from "svelte/transition";
    import Icon from "../Icon.svelte";
    import { cubicOut } from "svelte/easing";
    import type { Component } from "svelte";
    import { ArrowLeft } from "@lucide/svelte";
    import type { Track } from "$lib/types";

    interface MenuItem {
        label?: string;
        icon?: string;
        onClick?: () => void;
        href?: string;
        danger?: boolean;
        disabled?: boolean;
        type?: "separator";
        items?: MenuItem[];
        submenu?: Component;
        track?: Track;
    }

    interface DropdownMenuProps {
        items: MenuItem[];
        placement?: "bottom-left" | "bottom-right";
        onClose?: () => void;
    }

    let {
        items = [],
        placement = "bottom-right",
        onClose = () => {},
    }: DropdownMenuProps = $props();

    let subMenu = $state<MenuItem | null>(null);

    const handleClick = (item: MenuItem) => {
        if (item.submenu) {
            subMenu = item;
        } else if (item.onClick) {
            item.onClick();
        }
    };

    const style = $derived(
        placement === "bottom-left"
            ? "right-0 origin-top-right"
            : "left-0 origin-top-left",
    );
</script>

<svelte:document
    onmousedown={(e) => {
        const target = e.target as HTMLElement;
        if (
            !target.closest(".dropdown-menu") &&
            !target.closest(".dropdown-trigger")
        ) {
            onClose();
        }
    }}
/>

{#snippet MenuItem(item: MenuItem)}
    {#if item.type === "separator"}
        <div class="my-1.5 h-px bg-white/10" role="separator"></div>
    {:else}
        <div role="menu" tabindex="-1" class="relative">
            {#if item.href}
                <a
                    href={item.href}
                    role="menuitem"
                    class="flex items-center gap-3 rounded-xl px-3 py-2 text-zinc-200 transition-colors hover:bg-black/5 hover:text-white"
                    onclick={() => onClose()}
                >
                    {#if item.icon}<Icon
                            name={item.icon}
                            size={16}
                            class="shrink-0 text-zinc-400"
                        />{/if}
                    <span class="flex-1 truncate">{item.label}</span>
                </a>
            {:else}
                <button
                    type="button"
                    disabled={item.disabled}
                    role="menuitem"
                    class="flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left text-[13.5px] transition-colors disabled:opacity-40
          {item.danger
                        ? 'text-red-400 hover:bg-red-500/10 hover:text-red-300'
                        : 'text-zinc-200 hover:bg-gray-300/5 hover:text-white'}"
                    onclick={() => handleClick(item)}
                >
                    {#if item.icon}
                        <Icon
                            name={item.icon}
                            size={16}
                            class="shrink-0 {item.danger
                                ? 'text-red-400'
                                : 'text-zinc-400'}"
                        />
                    {/if}
                    <span class="flex-1 truncate">{item.label}</span>
                    {#if item.type === "submenu"}<Icon
                            name="chevron-right"
                            size={14}
                            class="shrink-0 text-zinc-500"
                        />{/if}
                </button>
            {/if}
        </div>
    {/if}
{/snippet}

<div
    class="dropdown-menu absolute z-50 min-w-55 max-w-75 rounded-2xl border border-white/10 bg-card/40 shadow-lg backdrop-blur-xl {style}"
    role="menu"
    transition:slide={{ duration: 200, easing: cubicOut }}
>
    {#if subMenu}
        <div class="flex flex-col">
            <div class="w-full bg-black/15 p-2 rounded-t-2xl flex items-center">
                <button onclick={() => (subMenu = null)}>
                    <ArrowLeft size={24} class="text-zinc-400" />
                </button>
            </div>
            <div class="p-2">
                <subMenu.submenu track={subMenu.track} />
            </div>
        </div>
    {:else}
        <div class="p-2">
            {#each items as item, i (i)}
                {@render MenuItem(item)}
            {/each}

            {#if items.length === 0}
                <p class="px-3 py-2 text-[13px] text-zinc-500">No options</p>
            {/if}
        </div>
    {/if}
</div>
