function binaryToOctet(value: number): number[] {
    return [(value >> 24) & 255, (value >> 16) & 255, (value >> 8) & 255, (value) & 255];
}

function octetToBinary(first: number, second: number, third: number, fourth: number): number {
    return (first << 24) + (second << 16) + (third << 8) + fourth;
}

function prettyOctet(octets: number[]): string {
    return `${octets[0]}.${octets[1]}.${octets[2]}.${octets[3]}`;
}

export default {
    binaryToOctet,
    octetToBinary,
    prettyOctet
}
