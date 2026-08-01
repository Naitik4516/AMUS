<script lang="ts">
    import GenreCard from "$components/ui/Card/GenreCard.svelte";
    import DisplayList from "$components/ui/DisplayList.svelte";
    import { createGenre } from "$lib/commands.svelte";
    import { store } from "$lib/stores.svelte";
    import { Plus } from "@lucide/svelte";
    import { goto } from "$app/navigation";
    import Dialog from "$components/Dialog.svelte";
    import Button from "$components/ui/button/button.svelte";

    let genres = $derived(store.genres);
    let showCreateDialog = $state(false);
    let newGenreName = $state("");

    async function handleCreate() {
        if (!newGenreName.trim()) return;
        const genre = await createGenre(newGenreName.trim());
        showCreateDialog = false;
        newGenreName = "";
        goto(`/library/genres/${genre.id}`);
    }
</script>

<svelte:head>
    <title>Genres</title>
</svelte:head>

<div class="relative">
    <div class="flex items-center justify-end px-8 pt-4">
        <Button
            onclick={() => (showCreateDialog = true)}
            variant="secondary"
            size="lg"
        >
            <Plus size={16} /> Create Genre
        </Button>
    </div>
    {#snippet Fallback()}
        <p class="text-gray-500 text-sm mb-4">No genres found.</p>
    {/snippet}
    <DisplayList
        listItems={genres}
        title="Genres"
        Card={GenreCard}
        fallBack={Fallback}
        cellHeight={370}
    />
</div>

<Dialog title="Create Genre" bind:open={showCreateDialog}>
    <input
        type="text"
        bind:value={newGenreName}
        placeholder="Genre name"
        class="w-full bg-black/30 border border-border rounded-xl px-4 py-2 text-sm text-white mb-4 outline-none focus:border-accent"
        onkeydown={(e) => e.key === "Enter" && handleCreate()}
    />

    {#snippet Footer()}
        <div class="flex justify-end gap-2">
            <Button onclick={() => (showCreateDialog = false)} variant="outline"
                >Cancel</Button
            >
            <Button onclick={handleCreate} disabled={!newGenreName.trim()}
                >Create</Button
            >
        </div>
    {/snippet}
</Dialog>
