import { BrowserRouter, Routes, Route, Link } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import ResolverList from './components/UpstreamResolvers/ResolverList';
import RuleList from './components/PriorityRules/RuleList';
import StatsDashboard from './components/Statistics/StatsDashboard';
import './App.css';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <div className="app">
          <header className="header">
            <h1>GOBLIN - DNS Redirector</h1>
            <p>Security Research Tool</p>
          </header>

          <div className="container">
            <nav className="nav">
              <Link to="/" className="nav-link">Dashboard</Link>
              <Link to="/resolvers" className="nav-link">Upstream Resolvers</Link>
              <Link to="/rules" className="nav-link">Priority Rules</Link>
            </nav>

            <main className="content">
              <Routes>
                <Route path="/" element={<StatsDashboard />} />
                <Route path="/resolvers" element={<ResolverList />} />
                <Route path="/rules" element={<RuleList />} />
              </Routes>
            </main>
          </div>

          <footer className="footer">
            <p>GOBLIN v0.1.0 - A DNS redirector for security researchers</p>
          </footer>
        </div>
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;
