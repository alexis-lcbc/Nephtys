<script lang="ts">
    let username = $state("");
    let password = $state("");

    async function login() {
        let result = await fetch('/api/auth/login', {
            method: 'POST',
            headers: {
            'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                username: username,
                password: password
            }),
            
        })
        if (result.ok) {
            alert("Successfully logged in!")
            window.location.href = "/";
        } else {
            alert("Failed to authenticate : " + await result.text())
        }
    }
</script>

<div class="bg-gray-900 text-white w-full h-full p-5">
    <div class="flex items-center justify-center flex-col">
        <h1 class="text-4xl pb-5">Log in to Nephtys</h1>
        <p class="p-2">Welcome back to your Nephtys instance!</p>
        <h2 class="text-4xl p-5">Log In</h2>
        <div class="flex items-stretch justify-stretch flex-col">
            <input class="bg-white text-black p-1 m-2 rounded-md" bind:value={username} type="text" placeholder="username">
            <input class="bg-white text-black p-1 m-2 rounded-md" bind:value={password} type="password" placeholder="password">
            <button class="bg-white text-black p-1 m-2 rounded-md" onclick={login}>Log In</button>
        </div>
    </div>
</div>
