<script lang="ts">
    let username = $state("");
    let password = $state("");

    async function send_signup() {
        let result = await fetch('/api/auth/create', {
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
            alert("Successfully signed up!")
            window.location.href = "/";
        } else {
            alert("Failed to authenticate : " + await result.text())
        }
    }
</script>

<div class="bg-gray-900 text-white w-full h-full p-5">
    <div class="flex items-center justify-center flex-col">
        <h1 class="text-4xl pb-5">Let's finish the setup!</h1>
        <p class="p-2">Welcome to your Nephtys instance! Let's create an account.<br>
        This account is stored locally and keeps your camera safe!</p>
        <h2 class="text-4xl p-5">Create an account</h2>
        <div class="flex items-stretch justify-stretch flex-col">
            <input class="bg-white text-black p-1 m-2 rounded-md" bind:value={username} type="text" placeholder="username">
            <input class="bg-white text-black p-1 m-2 rounded-md" bind:value={password} type="password" placeholder="password">
            <button class="bg-white text-black p-1 m-2 rounded-md" onclick={send_signup}>Create account</button>
        </div>
        <p class="p-5">The rest of the configuration is done through the configuration.yaml file in the server's working directory</p>

    </div>
</div>
