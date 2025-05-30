import { SetupService, Interface, Implementation, register } from '@platform/entity/service';
import { services } from '@register/services';
import { InvalidPluginEntity, PluginEntity, PluginRunData } from '@platform/types/bindings/plugins';

import * as Requests from '@platform/ipc/request/index';

@SetupService(services['plugins'])
export class Service extends Implementation {
    protected preload: PluginEntity[] = [];

    public override ready(): Promise<void> {
        return this.list()
            .installed()
            .then((plugins) => {
                this.preload = plugins;
            })
            .catch((err: Error) => {
                this.log().error(`Fail load installed plugins list: ${err.message}`);
            });
    }

    public list(): {
        installed(): Promise<PluginEntity[]>;
        invalid(): Promise<InvalidPluginEntity[]>;
        preload(): PluginEntity[];
    } {
        return {
            installed: (): Promise<PluginEntity[]> => {
                return new Promise((reslove, reject) => {
                    Requests.IpcRequest.send(
                        Requests.Plugins.ListInstalled.Response,
                        new Requests.Plugins.ListInstalled.Request(),
                    )
                        .then((response: Requests.Plugins.ListInstalled.Response) => {
                            reslove(response.plugins);
                        })
                        .catch(reject);
                });
            },
            invalid: (): Promise<InvalidPluginEntity[]> => {
                return new Promise((reslove, reject) => {
                    Requests.IpcRequest.send(
                        Requests.Plugins.ListInvalid.Response,
                        new Requests.Plugins.ListInvalid.Request(),
                    )
                        .then((response: Requests.Plugins.ListInvalid.Response) => {
                            reslove(response.invalidPlugins);
                        })
                        .catch(reject);
                });
            },
            preload: (): PluginEntity[] => {
                return this.preload;
            },
        };
    }

    public listInstalledPaths(): Promise<string[]> {
        return new Promise((reslove, reject) => {
            Requests.IpcRequest.send(
                Requests.Plugins.ListInstalledPaths.Response,
                new Requests.Plugins.ListInstalledPaths.Request(),
            )
                .then((response: Requests.Plugins.ListInstalledPaths.Response) => {
                    reslove(response.paths);
                })
                .catch(reject);
        });
    }

    public listInvalidPaths(): Promise<string[]> {
        return new Promise((reslove, reject) => {
            Requests.IpcRequest.send(
                Requests.Plugins.ListInvalidPaths.Response,
                new Requests.Plugins.ListInvalidPaths.Request(),
            )
                .then((response: Requests.Plugins.ListInvalidPaths.Response) => {
                    reslove(response.paths);
                })
                .catch(reject);
        });
    }

    public installedPluginInfo(pluginPath: string): Promise<PluginEntity | undefined> {
        return new Promise((reslove, reject) => {
            Requests.IpcRequest.send(
                Requests.Plugins.InstalledPluginInfo.Response,
                new Requests.Plugins.InstalledPluginInfo.Request({ pluginPath }),
            )
                .then((response: Requests.Plugins.InstalledPluginInfo.Response) => {
                    reslove(response.plugin);
                })
                .catch(reject);
        });
    }

    public invalidPluginInfo(pluginPath: string): Promise<InvalidPluginEntity | undefined> {
        return new Promise((reslove, reject) => {
            Requests.IpcRequest.send(
                Requests.Plugins.InvalidPluginInfo.Response,
                new Requests.Plugins.InvalidPluginInfo.Request({ pluginPath }),
            )
                .then((response: Requests.Plugins.InvalidPluginInfo.Response) => {
                    reslove(response.invalidPlugin);
                })
                .catch(reject);
        });
    }

    public getPluginRunData(pluginPath: string): Promise<PluginRunData | undefined> {
        return new Promise((reslove, reject) => {
            Requests.IpcRequest.send(
                Requests.Plugins.PluginRunData.Response,
                new Requests.Plugins.PluginRunData.Request({ pluginPath }),
            )
                .then((response: Requests.Plugins.PluginRunData.Response) => {
                    reslove(response.data);
                })
                .catch(reject);
        });
    }

    public reloadPlugins(): Promise<void> {
        return new Promise((resolve, reject) => {
            Requests.IpcRequest.send(
                Requests.Plugins.Reload.Response,
                new Requests.Plugins.Reload.Request(),
            )
                .then(() => {
                    // To drop a cache to updated list
                    this.ready().catch(reject).finally(resolve);
                })
                .catch(reject);
        });
    }

    public addPlugin(pluginPath: string): Promise<void> {
        return new Promise((resolve, reject) => {
            Requests.IpcRequest.send(
                Requests.Plugins.AddPlugin.Response,
                new Requests.Plugins.AddPlugin.Request({ pluginPath }),
            )
                .then(() => {
                    // To drop a cache to update list.
                    this.ready().catch(reject).finally(resolve);
                })
                .catch(reject);
        });
    }

    public removePlugin(pluginPath: string): Promise<void> {
        return new Promise((resolve, reject) => {
            Requests.IpcRequest.send(
                Requests.Plugins.RemovePlugin.Response,
                new Requests.Plugins.RemovePlugin.Request({ pluginPath }),
            )
                .then(() => {
                    // To drop a cache to update list.
                    this.ready().catch(reject).finally(resolve);
                })
                .catch(reject);
        });
    }
}

export interface Service extends Interface {}
export const plugins = register(new Service());
