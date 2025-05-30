import {
    Component,
    ChangeDetectorRef,
    AfterViewInit,
    Input,
    AfterContentInit,
} from '@angular/core';
import { Ilc, IlcInterface } from '@env/decorators/component';
import { Initial } from '@env/decorators/initial';
import { ChangesDetector } from '@ui/env/extentions/changes';
import { State } from '../../state';
import { State as ParserState } from '@ui/tabs/observe/parsers/state';

import * as Origins from '@platform/types/observe/origin/index';
import * as Parsers from '@platform/types/observe/parser/index';
@Component({
    selector: 'app-tabs-observe-concat',
    templateUrl: './template.html',
    styleUrls: ['./styles.less'],
    standalone: false,
})
@Initial()
@Ilc()
export class TabObserveConcat extends ChangesDetector implements AfterViewInit, AfterContentInit {
    @Input() state!: State;

    public parser!: ParserState;

    constructor(cdRef: ChangeDetectorRef) {
        super(cdRef);
    }

    public ngAfterContentInit(): void {
        const origin = this.state.observe.origin.as<Origins.Concat.Configuration>(
            Origins.Concat.Configuration,
        );
        if (origin === undefined) {
            throw new Error(`Current origin isn't a stream`);
        }
        this.parser = new ParserState(this.state.observe);
        this.env().subscriber.register(
            this.state.updates.get().parser.subscribe(() => {
                const parser = this.state.getParser().embedded();
                if (parser === undefined) {
                    return;
                }
                this.state.observe.parser.overwrite({
                    [parser]: Parsers.getByAlias(parser).configuration,
                });
            }),
        );
    }

    public ngAfterViewInit(): void {
        //
    }
}
export interface TabObserveConcat extends IlcInterface {}
