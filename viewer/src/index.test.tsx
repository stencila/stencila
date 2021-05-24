import { render } from 'solid-js/web';
import Viewer from './index';

it('renders without crashing', () => {
  const div = document.createElement('div');
  const dispose = render(Viewer, div);
  div.textContent = '';
  dispose();
});
