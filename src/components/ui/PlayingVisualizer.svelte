<span class="playing__bar playing__bar1"></span>
<span class="playing__bar playing__bar2"></span>
<span class="playing__bar playing__bar3"></span>

<style>
    .playing__bar {
        display: inline-block;
        background: var(--color-accent);
        width: 16%;
        height: 80%;
        animation: up-and-down 1.3s ease infinite alternate;
        border-radius: 12px;
    }

    .playing__bar1 {
        height: 60%;
    }

    .playing__bar2 {
        height: 30%;
        animation-delay: -2.2s;
    }

    .playing__bar3 {
        height: 75%;
        animation-delay: -3.7s;
    }

    @keyframes up-and-down {
        10% {
            height: 30%;
        }

        30% {
            height: 100%;
        }

        60% {
            height: 50%;
        }

        80% {
            height: 75%;
        }

        100% {
            height: 60%;
        }
    }
</style>
