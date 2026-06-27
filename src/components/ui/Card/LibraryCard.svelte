<script lang="ts">
    interface LibraryCardProps {
        label: string;
        Icon: any;
        href: string;
    }
    let { label, Icon, href }: LibraryCardProps = $props();
</script>

<a
    {href}
    class="flex gap-4 items-center w-full h-30 px-4 py-2 bg-white/5 border-3 border-secondary/20 hover:border-secondary/60 hover:bg-white/10 rounded-4xl transition-colors duration-300 cursor-pointer backdrop-blur-sm font-bold text-lg"
>
    <span>
        <Icon size={36} />
    </span>
    <div class="text-center">
        {label}
    </div>
</a>
