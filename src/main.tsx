import React from 'react';
import ReactDOM from 'react-dom/client';
import {PrivacyProvider} from "./Privacy";
import App from './App';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode><PrivacyProvider>{epoch=><App key={epoch}/>}</PrivacyProvider></React.StrictMode>,
);
