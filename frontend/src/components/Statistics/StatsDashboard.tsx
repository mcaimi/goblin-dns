import { useStatsSummary, useStats } from '../../hooks/useStats';
import { useResolvers } from '../../hooks/useResolvers';

export default function StatsDashboard() {
  const { data: summary, isLoading: summaryLoading } = useStatsSummary();
  const { data: stats, isLoading: statsLoading } = useStats(20, 0);
  const { data: resolvers } = useResolvers();

  if (summaryLoading) return <div className="loading">Loading statistics...</div>;

  const summaryData = summary?.data;
  const recentStats = stats?.data || [];

  return (
    <div>
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

      <div className="card">
        <div className="card-header">
          <h3 className="card-title">Recent Activity</h3>
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
