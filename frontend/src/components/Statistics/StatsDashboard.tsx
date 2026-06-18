import { useStatsSummary, useStats } from '../../hooks/useStats';

export default function StatsDashboard() {
  const { data: summary, isLoading: summaryLoading } = useStatsSummary();
  const { data: stats, isLoading: statsLoading } = useStats(20, 0);

  if (summaryLoading) return <div>Loading statistics...</div>;

  const summaryData = summary?.data;
  const recentStats = stats?.data || [];

  return (
    <div>
      <h2>Statistics Dashboard</h2>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '20px', marginBottom: '30px' }}>
        <div style={{ padding: '20px', border: '1px solid #ccc', borderRadius: '5px' }}>
          <h3 style={{ margin: '0 0 10px 0', color: '#666' }}>Total Hits</h3>
          <div style={{ fontSize: '32px', fontWeight: 'bold' }}>{summaryData?.total_hits || 0}</div>
        </div>

        <div style={{ padding: '20px', border: '1px solid #ccc', borderRadius: '5px' }}>
          <h3 style={{ margin: '0 0 10px 0', color: '#666' }}>Unique Clients</h3>
          <div style={{ fontSize: '32px', fontWeight: 'bold' }}>{summaryData?.unique_clients || 0}</div>
        </div>

        <div style={{ padding: '20px', border: '1px solid #ccc', borderRadius: '5px' }}>
          <h3 style={{ margin: '0 0 10px 0', color: '#666' }}>Active Rules</h3>
          <div style={{ fontSize: '32px', fontWeight: 'bold' }}>{summaryData?.active_rules || 0}</div>
        </div>
      </div>

      <h3>Recent Activity</h3>
      {statsLoading ? (
        <div>Loading recent activity...</div>
      ) : recentStats.length > 0 ? (
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ backgroundColor: '#f0f0f0' }}>
              <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Time</th>
              <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Domain</th>
              <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Client IP</th>
              <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Original IP</th>
              <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Overridden IP</th>
            </tr>
          </thead>
          <tbody>
            {recentStats.map((stat) => (
              <tr key={stat.id}>
                <td style={{ padding: '10px', border: '1px solid #ccc' }}>
                  {new Date(stat.timestamp * 1000).toLocaleString()}
                </td>
                <td style={{ padding: '10px', border: '1px solid #ccc' }}>{stat.requested_domain}</td>
                <td style={{ padding: '10px', border: '1px solid #ccc' }}>{stat.client_ip}</td>
                <td style={{ padding: '10px', border: '1px solid #ccc' }}>
                  {stat.original_ip || 'N/A'}
                </td>
                <td style={{ padding: '10px', border: '1px solid #ccc' }}>{stat.overridden_ip}</td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : (
        <div style={{ padding: '20px', textAlign: 'center', color: '#666' }}>
          No statistics recorded yet
        </div>
      )}
    </div>
  );
}
