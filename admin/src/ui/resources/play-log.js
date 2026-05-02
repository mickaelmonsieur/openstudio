export const playLogResource = {
  title: 'Play Log',
  endpoint: '/api/play-log',
  pageSize: 100,
  columns: [
    { key: 'id',              label: 'ID',       width: '80px'  },
    { key: 'played_at',       label: 'Played At', width: '180px' },
    { key: 'station',         label: 'Station',  width: '140px' },
    { key: 'artist',          label: 'Artist',   width: '180px' },
    { key: 'title',           label: 'Title'                    },
    { key: 'played_duration', label: 'Duration', width: '90px'  }
  ]
};
