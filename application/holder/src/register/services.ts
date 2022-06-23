import { v4 } from 'uuid';
import { Inputs } from 'platform/entity/service';

export const services: { [key: string]: Inputs } = {
    electron: {
        name: 'Electron',
        uuid: v4(),
    },
    paths: {
        name: 'Paths',
        uuid: v4(),
    },
    environment: {
        name: 'Environment',
        uuid: v4(),
    },
    production: {
        name: 'Production',
        uuid: v4(),
    },
    sessions: {
        name: 'Sessions',
        uuid: v4(),
    },
    jobs: {
        name: 'Jobs',
        uuid: v4(),
    },
    bridge: {
        name: 'Bridge',
        uuid: v4(),
    },
    storage: {
        name: 'Storage',
        uuid: v4(),
    },
};