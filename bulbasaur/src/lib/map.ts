const colorMap = {
    "unknown-*-*": {
        value: "#AAAAAA",
        display: "Unknown allocation state"
    },
    "reserved-*-*": {
        value: "#FF0000",
        display: "Reserved"
    },
    "unallocated-*-*": {
        value: "#0000FF",
        display: "Unallocated"
    },
    "allocated-false-false": {
        value: "#FFFF00",
        display: "Allocated unrouted"
    },
    "allocated-true-false": {
        value: "#00AA00",
        display: "Allocated routed offline"
    },
    "allocated-true-true": {
        value: "#00FF00",
        display: "Allocated online"
    },
};

function mapToColorId(allocation_state: string, routed: boolean, online: boolean): keyof typeof colorMap {
    switch (allocation_state) {
        case "unknown":
            return "unknown-*-*";

        case "reserved":
            return "reserved-*-*";

        case "unallocated":
            return "unallocated-*-*";

        case "allocated":
        default:
            return `allocated-${routed.toString()}-${online.toString()}` as keyof typeof colorMap;
    }
}

export default {
    colorMap,
    mapToColorId
}
