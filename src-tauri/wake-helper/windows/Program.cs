using System.Speech.Recognition;
using System.Text.Json;

const string DefaultEventFile = "jarvis-wake.jsonl";
var eventFile = GetArgument("--event-file") ?? Path.Combine(Path.GetTempPath(), DefaultEventFile);
var gate = new object();

void Emit(object payload)
{
    var line = JsonSerializer.Serialize(payload) + Environment.NewLine;
    lock (gate)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(eventFile) ?? ".");
        File.AppendAllText(eventFile, line);
    }
}

Emit(new { type = "boot" });

try
{
    var recognizers = SpeechRecognitionEngine.InstalledRecognizers()
        .Where(info => info.Culture.Name.StartsWith("zh-CN", StringComparison.OrdinalIgnoreCase)
                    || info.Culture.Name.StartsWith("en-US", StringComparison.OrdinalIgnoreCase))
        .ToList();

    if (recognizers.Count == 0)
    {
        Emit(new { type = "error", message = "Windows 未安装中文或英文语音识别器；请点击 Jarvis 麦克风按钮启动。" });
        return;
    }

    var engines = new List<SpeechRecognitionEngine>();
    foreach (var info in recognizers)
    {
        var engine = new SpeechRecognitionEngine(info);
        var grammar = new Choices(
            "嘿，杰克", "嘿 杰克", "嘿杰克", "Hey Jack", "Hey Jake");
        engine.LoadGrammar(new Grammar(new GrammarBuilder(grammar)));
        engine.SpeechRecognized += (_, args) =>
        {
            if (args.Result.Confidence >= 0.55)
            {
                Emit(new { type = "wake", phrase = args.Result.Text });
            }
        };
        engine.SetInputToDefaultAudioDevice();
        engine.RecognizeAsync(RecognizeMode.Multiple);
        engines.Add(engine);
    }

    Emit(new { type = "authorization", status = "authorized" });
    Emit(new { type = "ready" });
    using var quit = new ManualResetEventSlim(false);
    Console.CancelKeyPress += (_, args) =>
    {
        args.Cancel = true;
        quit.Set();
    };
    quit.Wait();
    foreach (var engine in engines)
    {
        engine.RecognizeAsyncStop();
        engine.Dispose();
    }
}
catch (PlatformNotSupportedException error)
{
    Emit(new { type = "error", message = "Windows 语音识别不可用：" + error.Message });
}
catch (InvalidOperationException error)
{
    Emit(new { type = "error", message = "Windows 麦克风不可用：" + error.Message });
}
catch (Exception error)
{
    Emit(new { type = "error", message = "唤醒助手启动失败：" + error.Message });
}

static string? GetArgument(string name)
{
    var args = Environment.GetCommandLineArgs();
    var index = Array.IndexOf(args, name);
    return index >= 0 && index + 1 < args.Length ? args[index + 1] : null;
}
