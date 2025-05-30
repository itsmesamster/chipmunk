import { CancelablePromise } from 'platform/env/promise';
import { Logger } from 'platform/log';
import { unbound } from '@service/unbound';

import * as Requests from 'platform/ipc/request';

export const handler = Requests.InjectLogger<
    Requests.Plugins.ListInvalid.Request,
    CancelablePromise<Requests.Plugins.ListInvalid.Response>
>(
    (
        _log: Logger,
        _request: Requests.Plugins.ListInvalid.Request,
    ): CancelablePromise<Requests.Plugins.ListInvalid.Response> => {
        return new CancelablePromise((reslove, reject) => {
            unbound.jobs
                .invalidPluginsList()
                .then((invalidPlugins) => {
                    reslove(new Requests.Plugins.ListInvalid.Response({ invalidPlugins }));
                })
                .catch(reject);
        });
    },
);
