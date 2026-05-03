export const usersResource = {
  name: 'users',
  title: 'Users',
  group: 'Admin',
  endpoint: '/api/users',
  primaryKey: 'id',
  columns: [
    { key: 'id', label: 'ID', width: '90px' },
    { key: 'login', label: 'Login' },
    { key: 'role_name', label: 'Role', width: '130px' },
    { key: 'active', label: 'Active', width: '90px' }
  ],
  form: [
    { key: 'login', label: 'Login', type: 'text', required: true, maxLength: 32 },
    { key: 'password', label: 'Password (laisser vide pour conserver)', type: 'password', required: false },
    {
      key: 'role_id',
      label: 'Role',
      type: 'select',
      required: true,
      options: [
        { value: 1, label: 'SuperAdmin' },
        { value: 2, label: 'Admin' },
        { value: 3, label: 'Manager' },
        { value: 4, label: 'User' }
      ]
    },
    { key: 'active', label: 'Active', type: 'checkbox' }
  ],
  actions: {
    add: true,
    edit: true,
    delete: true
  }
};
