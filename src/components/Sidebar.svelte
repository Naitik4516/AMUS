<script>
    import { House, Bookmark, Settings, Heart } from "@lucide/svelte";

    const floating_bar =
        "floating_bar flex flex-col items-center gap-10 p-6 rounded-full surface text-white shadow-lg ";
</script>

<div
    class="sidebar h-full md:h-[calc(90vh-130px)] left-2 flex flex-col items-center justify-between py-4 mr-5"
>
    <div class={floating_bar}>
        <a title="Home" href="/">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                height="26px"
                viewBox="0 -960 960 960"
                fill="#e3e3e3"
                ><path
                    d="M160-200v-360q0-19 8.5-36t23.5-28l240-180q21-16 48-16t48 16l240 180q15 11 23.5 28t8.5 36v360q0 33-23.5 56.5T720-120H600q-17 0-28.5-11.5T560-160v-200q0-17-11.5-28.5T520-400h-80q-17 0-28.5 11.5T400-360v200q0 17-11.5 28.5T360-120H240q-33 0-56.5-23.5T160-200Z"
                /></svg
            >
        </a>
        <a title="Library" href="/library">
            <Bookmark size={24} fill="white" />
        </a>
        <a title="Favorites" href="/library/favourites">
            <Heart size={23} fill="white" />
        </a>
        <a title="Artists" href="/library/artists">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                height="24px"
                viewBox="0 -960 960 960"
                width="24px"
                fill="#e3e3e3"
                ><path
                    d="M629-189q-29-29-29-71t29-71q29-29 71-29 8 0 18 1.5t22 6.5v-168q0-17 11.5-28.5T780-560h60q17 0 28.5 11.5T880-520q0 17-11.5 28.5T840-480h-40v220q0 42-29 71t-71 29q-42 0-71-29ZM327-527q-47-47-47-113t47-113q47-47 113-47t113 47q47 47 47 113t-47 113q-47 47-113 47t-113-47ZM160-160q-17 0-28.5-11.5T120-200v-72q0-35 17.5-63t46.5-43q62-31 126-46.5T440-440q28 0 55.5 3t55.5 9q17 4 21.5 21t-9.5 31q-21 25-31.5 54.5T521-260q0 13 1.5 25.5T528-209q5 18-4.5 33.5T497-160H160Z"
                /></svg
            >
        </a>
    </div>
    <div class={floating_bar}>
        <a title="Settings" href="/settings">
            <Settings size={24} />
        </a>
    </div>
</div>

<style>
    .floating_bar button {
        transition: color 0.3s ease;
        cursor: pointer;
    }
    .floating_bar button:hover {
        color: var(--color-secondary);
    }
</style>
