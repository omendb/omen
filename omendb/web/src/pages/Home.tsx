import { Component } from 'solid-js';
import Hero from '../components/Hero';
import Features from '../components/Features';
import Performance from '../components/Performance';
import BenchmarkComparison from '../components/BenchmarkComparison';
import CodeExample from '../components/CodeExample';
import CTA from '../components/CTA';

const Home: Component = () => {
  return (
    <>
      <Hero />
      <Features />
      <Performance />
      <BenchmarkComparison />
      <CodeExample />
      <CTA />
    </>
  );
};

export default Home;