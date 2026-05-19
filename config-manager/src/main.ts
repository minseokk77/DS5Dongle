import './styles.css';
import App from './App.svelte';
import { mount } from 'svelte';

const target = document.getElementById('app');

if (!target) {
  throw new Error('앱을 마운트할 #app 요소를 찾을 수 없습니다.');
}

const app = mount(App, { target });

export default app;
