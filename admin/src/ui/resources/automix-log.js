export const automixLogResource = {
  title: 'Auto Mix Log',
  endpoint: '/api/automix-log',
  pageSize: 100,
  columns: [
    { key: 'id',        label: 'ID',        width: '80px'  },
    { key: 'logged_at', label: 'Logged At', width: '180px' },
    { key: 'message',   label: 'Message'                   }
  ]
};
