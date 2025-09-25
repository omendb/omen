/* @refresh reload */
import { render } from 'solid-js/web';
import { Router, Route } from '@solidjs/router';
import { lazy } from 'solid-js';
import './index.css';

// Lazy load pages for better performance
const Home = lazy(() => import('./pages/Home'));
const Docs = lazy(() => import('./pages/Docs'));

import App from './App';

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got mispelled?',
  );
}

render(() => (
  <Router root={App}>
    <Route path="/" component={Home} />
    <Route path="/docs" component={Docs} />
  </Router>
), root!);