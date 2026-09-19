import React from 'react';
import ReactDOM from 'react-dom/client';
import {PrivacyProvider} from "./Privacy";
import App from './App';
import WindowFrame from './WindowFrame';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode><WindowFrame><PrivacyProvider>{epoch=><App key={epoch}/>}</PrivacyProvider></WindowFrame></React.StrictMode>,
);
