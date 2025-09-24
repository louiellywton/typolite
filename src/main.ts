import App from './App.svelte';
import './app.css';

// Remove loading spinner
const loading = document.getElementById('loading');
if (loading) {
  loading.remove();
}

const app = new App({
  target: document.getElementById('app')!,
});

export default app;
