import { useState } from 'react';
import { useRules, useCreateRule, useDeleteRule, useToggleRule } from '../../hooks/useRules';
import type { CreateRuleRequest } from '../../types';

export default function RuleList() {
  const { data, isLoading, error } = useRules();
  const createMutation = useCreateRule();
  const deleteMutation = useDeleteRule();
  const toggleMutation = useToggleRule();

  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState<CreateRuleRequest>({
    domain: '',
    client_filter: '',
    target_ip: '',
    active: true,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const data = {
      ...formData,
      client_filter: formData.client_filter || null,
    };
    await createMutation.mutateAsync(data);
    setFormData({ domain: '', client_filter: '', target_ip: '', active: true });
    setShowForm(false);
  };

  const handleDelete = async (id: number) => {
    if (confirm('Are you sure you want to delete this rule?')) {
      await deleteMutation.mutateAsync(id);
    }
  };

  const handleToggle = async (id: number) => {
    await toggleMutation.mutateAsync(id);
  };

  if (isLoading) return <div>Loading rules...</div>;
  if (error) return <div>Error loading rules: {error.message}</div>;

  const rules = data?.data || [];

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '20px' }}>
        <h2>Priority Rules</h2>
        <button onClick={() => setShowForm(!showForm)}>
          {showForm ? 'Cancel' : 'Add Rule'}
        </button>
      </div>

      {showForm && (
        <form onSubmit={handleSubmit} style={{ marginBottom: '20px', padding: '15px', border: '1px solid #ccc' }}>
          <div style={{ marginBottom: '10px' }}>
            <label>
              Domain:
              <input
                type="text"
                value={formData.domain}
                onChange={(e) => setFormData({ ...formData, domain: e.target.value })}
                placeholder="evil.com"
                required
                style={{ marginLeft: '10px', padding: '5px', width: '300px' }}
              />
            </label>
          </div>
          <div style={{ marginBottom: '10px' }}>
            <label>
              Client Filter (optional):
              <input
                type="text"
                value={formData.client_filter || ''}
                onChange={(e) => setFormData({ ...formData, client_filter: e.target.value })}
                placeholder="192.168.1.0/24 or empty for all"
                style={{ marginLeft: '10px', padding: '5px', width: '300px' }}
              />
            </label>
          </div>
          <div style={{ marginBottom: '10px' }}>
            <label>
              Target IP:
              <input
                type="text"
                value={formData.target_ip}
                onChange={(e) => setFormData({ ...formData, target_ip: e.target.value })}
                placeholder="127.0.0.1"
                required
                style={{ marginLeft: '10px', padding: '5px', width: '300px' }}
              />
            </label>
          </div>
          <div style={{ marginBottom: '10px' }}>
            <label>
              <input
                type="checkbox"
                checked={formData.active}
                onChange={(e) => setFormData({ ...formData, active: e.target.checked })}
              />
              {' '}Active
            </label>
          </div>
          <button type="submit" disabled={createMutation.isPending}>
            {createMutation.isPending ? 'Creating...' : 'Create Rule'}
          </button>
        </form>
      )}

      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr style={{ backgroundColor: '#f0f0f0' }}>
            <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>ID</th>
            <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Domain</th>
            <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Client Filter</th>
            <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Target IP</th>
            <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Status</th>
            <th style={{ padding: '10px', textAlign: 'left', border: '1px solid #ccc' }}>Actions</th>
          </tr>
        </thead>
        <tbody>
          {rules.map((rule) => (
            <tr key={rule.id} style={{ backgroundColor: rule.active ? 'white' : '#f9f9f9' }}>
              <td style={{ padding: '10px', border: '1px solid #ccc' }}>{rule.id}</td>
              <td style={{ padding: '10px', border: '1px solid #ccc' }}>{rule.domain}</td>
              <td style={{ padding: '10px', border: '1px solid #ccc' }}>
                {rule.client_filter || 'All clients'}
              </td>
              <td style={{ padding: '10px', border: '1px solid #ccc' }}>{rule.target_ip}</td>
              <td style={{ padding: '10px', border: '1px solid #ccc' }}>
                <span style={{ color: rule.active ? 'green' : 'gray' }}>
                  {rule.active ? 'Active' : 'Inactive'}
                </span>
              </td>
              <td style={{ padding: '10px', border: '1px solid #ccc' }}>
                <button onClick={() => handleToggle(rule.id)} style={{ marginRight: '10px' }}>
                  {rule.active ? 'Deactivate' : 'Activate'}
                </button>
                <button onClick={() => handleDelete(rule.id)} style={{ color: 'red' }}>
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {rules.length === 0 && (
        <div style={{ padding: '20px', textAlign: 'center', color: '#666' }}>
          No priority rules configured
        </div>
      )}
    </div>
  );
}
