import { Component } from 'solid-js';
import Navbar from './components/Navbar';
import Footer from './components/Footer';

const App: Component = (props) => {
  return (
    <div class="min-h-screen flex flex-col bg-white dark:bg-gray-900">
      <Navbar />
      <main class="flex-1">
        {props.children}
      </main>
      <Footer />
    </div>
  );
};

export default App;