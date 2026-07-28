<script lang="ts">
    import { Play } from "@lucide/svelte";
    import PlaylistCoverArt from "$components/ui/PlaylistCoverArt.svelte";
    import PlaylistCardMenu from "$components/ui/Menu/PlaylistCardMenu.svelte";
    import EditPlaylistDialog from "$components/ui/Dialog/EditPlaylistDialog.svelte";
    import ConfirmDialog from "$components/ui/Dialog/ConfirmDialog.svelte";
    import { store } from "$lib/stores.svelte";
    import { openContextMenu } from "$lib/context-menu.svelte";

    let { data } = $props();

    let editDialogOpen = $state(false);
    let deleteDialogOpen = $state(false);

    function handleContextMenu(e: MouseEvent) {
        e.preventDefault();
        openContextMenu(PlaylistCardMenu, {
            position: { type: "coordinates", x: e.clientX, y: e.clientY },
            playlist: data,
            onEdit: () => {
                editDialogOpen = true;
            },
            onDelete: () => {
                deleteDialogOpen = true;
            },
        });
    }

    async function handleDelete() {
        try {
            await store.deletePlaylist(data.id);
        } catch (e) {
            console.error("Failed to delete playlist", e);
        }
    }
</script>

<a
    href="/library/playlists/{data.id}?data.name={data.name}"
    oncontextmenu={handleContextMenu}
    class="group flex flex-col gap-3 p-6 rounded-3xl bg-card/80 transition-all duration-300 ring-2 ring-zinc-800/70 hover:ring-3 w-64 h-auto shadow-xl hover:shadow-card"
>
    <div
        class="aspect-square w-full rounded-3xl overflow-hidden relative inset-shadow-sm"
    >
        <PlaylistCoverArt playlist={data} />

        <div
            class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
        >
            <div
                class="bg-accent text-black p-4 rounded-full shadow-xl transform translate-y-4 group-hover:translate-y-0 transition-transform"
            >
                <Play size={20} fill="var(--color-accent-foreground)" />
            </div>
        </div>
    </div>

    <div class="flex flex-col mt-2">
        <h3 class="font-bold font-satoshi truncate text-white text-lg">
            {data.name}
        </h3>
    </div>
</a>

<EditPlaylistDialog
    bind:open={editDialogOpen}
    playlistId={data.id}
    name={data.name}
    coverArt={data.cover_art}
/>

<ConfirmDialog
    bind:open={deleteDialogOpen}
    title="Delete playlist"
    message={`Are you sure you want to delete "${data.name}"? This action cannot be undone.`}
    confirmLabel="Delete"
    onConfirm={handleDelete}
/>
