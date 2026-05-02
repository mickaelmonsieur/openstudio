export const stationsResource = {
  name: 'stations',
  title: 'Stations',
  endpoint: '/api/stations',
  primaryKey: 'id',
  columns: [
    { key: 'id', label: 'ID', width: '90px' },
    { key: 'name', label: 'Name' }
  ],
  form: [
    {
      key: 'name',
      label: 'Name',
      type: 'text',
      required: true,
      maxLength: 64
    }
  ],
  actions: {
    add: true,
    edit: true,
    delete: true
  }
};
