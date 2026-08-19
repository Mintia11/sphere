struct PushConstants
{
    float2 screen_size;
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> pushConstants;

[[vk::binding(0, 0)]]
Sampler2D font_texture;

struct VertexOutput
{
    float4 sv_position : SV_Position;
    [[vk::location(0)]]
    float4 color : COLOR0;
    [[vk::location(1)]]
    float2 uv : TEXCOORD0;
};

struct VertexInput
{
    [[vk::location(0)]]
    float2 pos : POSITION;
    [[vk::location(1)]]
    float2 uv : TEXCOORD0;
    [[vk::location(2)]]
    float4 color : COLOR0;
};

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    VertexOutput output;

    output.sv_position = float4(
        2.0 * input.pos.x / pushConstants.screen_size.x - 1.0,
        2.0 * input.pos.y / pushConstants.screen_size.y - 1.0,
        0.0,
        1.0);

    output.color = input.color;
    output.uv = input.uv;

    return output;
}

struct FragmentOutput
{
    float4 color : SV_Target;
};

[shader("fragment")]
FragmentOutput fragmentMain(VertexOutput input)
{
    FragmentOutput output;
    output.color = input.color * font_texture.Sample(input.uv);
    return output;
}
