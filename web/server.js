import express from 'express';
import { createProxyMiddleware } from 'http-proxy-middleware';
import { handler } from './build/handler.js';

const app = express();

// Proxy configuration
app.use(
    '/api',
    createProxyMiddleware({
        target: 'http://localhost:8080',
        changeOrigin: true,
        pathRewrite: {
            '^/api': ''
        }
    })
);

// SvelteKit's handler as a middleware
app.use(handler);

// Start the server
app.listen(3000, () => {
    console.log('Server is running on http://localhost:3000');
});