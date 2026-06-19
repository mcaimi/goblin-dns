import { useState } from 'react';
import { useStatsSummary, useStats } from '../../hooks/useStats';
import { useResolvers } from '../../hooks/useResolvers';

export default function StatsDashboard() {
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [refreshInterval, setRefreshInterval] = useState(5);

  const { data: summary, isLoading: summaryLoading } = useStatsSummary();
  const { data: stats, isLoading: statsLoading } = useStats(20, 0, autoRefresh ? refreshInterval * 1000 : false);
  const { data: resolvers } = useResolvers();

  const summaryData = summary?.data;
  const recentStats = stats?.data || [];

  return (
    <div>
    { summaryLoading ? 
    (
      <div className="loading">Loading statistics...</div>
    ) : (
      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-label">Total Hits</div>
          <div className="stat-value">{summaryData?.total_hits || 0}</div>
        </div>

        <div className="stat-card">
          <div className="stat-label">Unique Clients</div>
          <div className="stat-value">{summaryData?.unique_clients || 0}</div>
        </div>

        <div className="stat-card">
          <div className="stat-label">Active Rules</div>
          <div className="stat-value">{summaryData?.active_rules || 0}</div>
        </div>

        <div className="stat-card">
          <div className="stat-label">Total Resolvers</div>
          <div className="stat-value">{resolvers?.data?.length || 0}</div>
        </div>
      </div> 
    )
    }

      <div className="card">
        <div className="card-header">
          <h3 className="card-title">Recent Activity</h3>
          <div style={{ display: 'flex', alignItems: 'center', gap: '15px' }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={autoRefresh}
                onChange={(e) => setAutoRefresh(e.target.checked)}
                style={{ cursor: 'pointer' }}
              />
              <span>Auto-refresh</span>
            </label>
            {autoRefresh && (
              <select
                value={refreshInterval}
                onChange={(e) => setRefreshInterval(Number(e.target.value))}
                style={{
                  padding: '4px 8px',
                  borderRadius: '4px',
                  border: '1px solid #ddd',
                  backgroundColor: 'white',
                  cursor: 'pointer'
                }}
              >
                <option value={3}>Every 3s</option>
                <option value={5}>Every 5s</option>
                <option value={10}>Every 10s</option>
                <option value={30}>Every 30s</option>
                <option value={60}>Every 1m</option>
              </select>
            )}
          </div>
        </div>
        {statsLoading ? (
          <div className="loading">Loading recent activity...</div>
        ) : recentStats.length > 0 ? (
          <table>
            <thead>
              <tr>
                <th>Time</th>
                <th>Domain</th>
                <th>Client IP</th>
                <th>Original IP</th>
                <th>Overridden IP</th>
              </tr>
            </thead>
            <tbody>
              {recentStats.map((stat) => (
                <tr key={stat.id}>
                  <td>{new Date(stat.timestamp * 1000).toLocaleString()}</td>
                  <td>{stat.requested_domain}</td>
                  <td>{stat.client_ip}</td>
                  <td>{stat.original_ip || 'N/A'}</td>
                  <td>{stat.overridden_ip}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <p style={{ textAlign: 'center', color: '#95a5a6', padding: '20px' }}>
            No statistics recorded yet
          </p>
        )}
      </div>
    </div>
  );
}
