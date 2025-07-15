import React from 'react';
import styled, { createGlobalStyle } from 'styled-components';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import { Dashboard } from './pages/Dashboard';
import { AgentEditor } from './pages/AgentEditor';
import { ArenaViewer } from './pages/ArenaViewer';
import { Training } from './pages/Training';
import { Sidebar } from './components/Sidebar';
import { Header } from './components/Header';

const GlobalStyle = createGlobalStyle`
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  body {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    background: linear-gradient(135deg, #1e1e2e 0%, #2d2d44 100%);
    color: #e4e4e7;
    overflow-x: hidden;
  }

  ::-webkit-scrollbar {
    width: 8px;
  }

  ::-webkit-scrollbar-track {
    background: #27272a;
  }

  ::-webkit-scrollbar-thumb {
    background: #3f3f46;
    border-radius: 4px;
  }

  ::-webkit-scrollbar-thumb:hover {
    background: #52525b;
  }
`;

const AppContainer = styled.div`
  display: flex;
  height: 100vh;
  background: #18181b;
`;

const MainContent = styled.div`
  flex: 1;
  display: flex;
  flex-direction: column;
  margin-left: 240px;
`;

const ContentArea = styled.div`
  flex: 1;
  padding: 20px;
  overflow-y: auto;
  background: linear-gradient(135deg, #1e1e2e 0%, #2d2d44 100%);
`;

function App() {
  return (
    <Router>
      <GlobalStyle />
      <AppContainer>
        <Sidebar />
        <MainContent>
          <Header />
          <ContentArea>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/agents" element={<AgentEditor />} />
              <Route path="/arena" element={<ArenaViewer />} />
              <Route path="/training" element={<Training />} />
            </Routes>
          </ContentArea>
        </MainContent>
        <Toaster 
          position="bottom-right"
          toastOptions={{
            style: {
              background: '#27272a',
              color: '#e4e4e7',
              border: '1px solid #3f3f46',
            },
          }}
        />
      </AppContainer>
    </Router>
  );
}

export default App;