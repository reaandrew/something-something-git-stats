import {h} from 'preact';
import * as echarts from "echarts";
import {useEffect} from "preact/compat";
import LinesAddedDeleted from "../lines_added_deleted";
import LinesCumulative from "../lines_cumulative";
import LinesAddedDeletedSummary from "../lines_added_deleted_summary";


export default function Lines({data}) {
    return (
        <div>
            <div className="row">
                <div className="col">
                    <LinesAddedDeleted data={data}/>
                </div>
            </div>
            <div className="row">
                <div className="col">
                    <LinesCumulative data={data}/>
                </div>
                <div className="col-3">
                    <LinesAddedDeletedSummary data={data} />
                </div>
            </div>
        </div>
    )
}