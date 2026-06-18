import { BrowserRouter, Routes, Route, NavLink } from 'react-router-dom';
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
          <nav className="navbar">
            <div className="navbar-brand">
              <h1>GOBLIN - DNS Redirector</h1>
              <span className="navbar-subtitle">Security Research Tool</span>
            </div>
            <div className="navbar-menu">
              <NavLink to="/" className={({isActive}) => isActive ? 'nav-link active' : 'nav-link'}>
                Dashboard
              </NavLink>
              <NavLink to="/resolvers" className={({isActive}) => isActive ? 'nav-link active' : 'nav-link'}>
                Upstream Resolvers
              </NavLink>
              <NavLink to="/rules" className={({isActive}) => isActive ? 'nav-link active' : 'nav-link'}>
                Priority Rules
              </NavLink>
            </div>
          </nav>

          <main className="main-content">
            <Routes>
              <Route path="/" element={<StatsDashboard />} />
              <Route path="/resolvers" element={<ResolverList />} />
              <Route path="/rules" element={<RuleList />} />
            </Routes>
          </main>

          <footer className="footer">
            <p>GOBLIN v0.1.0 - A DNS redirector for security researchers</p>
          </footer>
        </div>
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;
