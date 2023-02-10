import checker from 'vite-plugin-checker';

export default {
  plugins: [
    checker({
      // e.g. use TypeScript check
      typescript: true,
    }),
  ],
}
