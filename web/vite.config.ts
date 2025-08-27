import devtoolsJson from 'vite-plugin-devtools-json';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit(), devtoolsJson()],
	server: {
        proxy: {
            // Proxy all requests starting with '/api' to your Actix server
            '/api': {
                target: 'http://127.0.0.1:8080',
                changeOrigin: true,
                // Rewrite the path if needed (e.g., to remove '/api')
                rewrite: (path) => path.replace(/^\/api/, '')
            },
        },
    },
});
