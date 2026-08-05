<script lang="ts">
    import { slide } from "svelte/transition";
    import Icon from "../Icon.svelte";
    import { cubicOut } from "svelte/easing";
    import type { Component } from "svelte";
    import { ArrowLeft } from "@lucide/svelte";
    import type { Track, MenuPosition, Context } from "$lib/types";

    export interface MenuItem {
        label?: string;
        icon?: string;
        onClick?: () => void;
        href?: string;
        danger?: boolean;
        disabled?: boolean;
        type?: "separator";
        items?: MenuItem[];
        submenu?: Component<any>;
        track?: Track;
        tracks?: Track[];
        context?: Context;
    }

    interface DropdownMenuProps {
        items: MenuItem[];
        position: MenuPosition;
        onClose?: () => void;
    }

    let {
        items = [],
        position,
        onClose = () => {},
    }: DropdownMenuProps = $props();

    let subMenu = $state<MenuItem | null>(null);

    let menuWidth = $state(0);
    let menuHeight = $state(0);

    let menuEl = $state<HTMLDivElement | null>(null);

    let { x, y } = $derived.by(() => {
        let x, y;
        if (position.type === "coordinates") {
            x = position.x;
            y = position.y;
        } else {
            const rect = position.anchor.getBoundingClientRect();
            x = rect.left;
            y = rect.bottom;
        }

        return {
            x: Math.min(x, window.innerWidth - menuWidth),
            y: Math.min(y, window.innerHeight - menuHeight),
        };
    });

    const handleClick = (item: MenuItem) => {
        if (item.submenu) {
            subMenu = item;
        } else if (item.onClick) {
            item.onClick();
            onClose();
        }
    };

    $effect(() => {
        const el = menuEl;
        if (!el) return;
        el.querySelector<HTMLElement>('[role="menuitem"]')?.focus({
            preventScroll: true,
        });

        const onKey = (e: KeyboardEvent) => {
            if (e.key === "Escape" || e.key === "Tab") {
                onClose();
                return;
            }
            if (e.key === "ArrowDown" || e.key === "ArrowUp") {
                e.preventDefault();
                const items = [
                    ...el.querySelectorAll<HTMLElement>('[role="menuitem"]'),
                ].filter((el) => !(el as HTMLButtonElement).disabled);
                if (items.length === 0) return;
                const current = document.activeElement;
                const idx = items.indexOf(current as HTMLElement);
                const next = e.key === "ArrowDown" ? idx + 1 : idx - 1;
                const target =
                    next > items.length - 1 ? 0 : next < 0 ? items.length - 1 : next;
                items[target]?.focus();
            }
        };
        el.addEventListener("keydown", onKey);
        return () => el.removeEventListener("keydown", onKey);
    });
</script>

<svelte:document
    onmousedown={(e) => {
        if (e.button === 2) return;
        const target = e.target as HTMLElement;
        if (
            !target.closest(".dropdown-menu") &&
            !target.closest(".dropdown-trigger")
        ) {
            onClose();
        }
    }}
    onscroll={(e) => {
        const target = e.target as HTMLElement | Document;
        if (target instanceof Element && target.closest(".dropdown-menu")) {
            return;
        }
        onClose();
    }}
/>

{#snippet MenuItem(item: MenuItem)}
    {#if item.type === "separator"}
        <div class="my-1.5 h-px bg-white/10" role="separator"></div>
    {:else}
        <div class="relative">
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
    class="dropdown-menu fixed z-100 min-w-55 max-w-75 rounded-2xl border border-white/10 bg-white/5  shadow-lg backdrop-blur-xl"
    bind:this={menuEl}
    bind:offsetWidth={menuWidth}
    bind:offsetHeight={menuHeight}
    style="top: {y}px; left: {x}px;"
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
                <subMenu.submenu
                    track={subMenu.track}
                    tracks={subMenu.tracks}
                    context={subMenu.context}
                />
            </div>
        </div>
    {:else}
        <div class="p-2" role="menu">
            {#each items as item, i (i)}
                {@render MenuItem(item)}
            {/each}

            {#if items.length === 0}
                <p class="px-3 py-2 text-[13px] text-zinc-500">No options</p>
            {/if}
        </div>
    {/if}
</div>
