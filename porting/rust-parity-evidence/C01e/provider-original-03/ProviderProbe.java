// Development-only oracle over the actual selected provider classes/resources.
import java.lang.reflect.Proxy;
import java.net.URL;
import org.eclipse.syson.sysml.*;
import org.eclipse.syson.sysml.provider.*;
import org.eclipse.syson.util.IDescriptionNameGenerator;
import org.eclipse.sirius.components.view.ViewFactory;
import org.eclipse.sirius.components.view.builder.providers.IColorProvider;

public class ProviderProbe {
    public static void main(String[] args) throws Exception {
        boolean derived = Boolean.parseBoolean(args[0]);
        var factory = new SysmlItemProviderAdapterFactory();
        int count = 0;
        for (String kind : derived ? new String[]{"ItemUsage", "ReferenceUsage"} : new String[]{"ReferenceUsage"}) {
            for (String owner : new String[]{"Step", "Behavior", "PartDefinition"}) {
                for (FeatureDirectionKind direction : new FeatureDirectionKind[]{null,
                        FeatureDirectionKind.IN, FeatureDirectionKind.OUT, FeatureDirectionKind.INOUT}) {
                    Feature feature = kind.equals("ItemUsage") ? SysmlFactory.eINSTANCE.createItemUsage()
                            : SysmlFactory.eINSTANCE.createReferenceUsage();
                    Type type = switch (owner) {
                        case "Step" -> SysmlFactory.eINSTANCE.createStep();
                        case "Behavior" -> SysmlFactory.eINSTANCE.createBehavior();
                        default -> SysmlFactory.eINSTANCE.createPartDefinition();
                    };
                    var membership = SysmlFactory.eINSTANCE.createFeatureMembership();
                    type.getOwnedRelationship().add(membership);
                    membership.getOwnedRelatedElement().add(feature);
                    feature.setDirection(direction);
                    Object image = kind.equals("ItemUsage")
                            ? new ItemUsageItemProvider(factory).getImage(feature)
                            : new ReferenceUsageItemProvider(factory).getImage(feature);
                    String suffix = direction == null ? "" : switch(direction) {
                        case IN -> "In"; case OUT -> "Out"; case INOUT -> "Inout";
                    };
                    String base = derived && direction != null && !owner.equals("PartDefinition")
                            ? "parameter" : kind;
                    if (!derived && kind.equals("ReferenceUsage")) suffix = "";
                    String expected = "/icons/full/obj16/" + base + suffix + ".svg";
                    if (!(image instanceof URL url) || !url.toString().endsWith(expected))
                        throw new AssertionError(kind + "/" + owner + "/" + direction + ": " + image + " expected " + expected);
                    try (var stream = ((URL) image).openStream()) {
                        if (stream.readAllBytes().length == 0) throw new AssertionError("empty icon");
                    }
                    System.out.println(kind + "\t" + owner + "\t" + direction + "\t" + expected);
                    count++;
                }
            }
        }
        IColorProvider colors = name -> {
            var color = ViewFactory.eINSTANCE.createFixedColor(); color.setName(name); return color;
        };
        var names = (IDescriptionNameGenerator) Proxy.newProxyInstance(
                ProviderProbe.class.getClassLoader(), new java.lang.Class<?>[]{IDescriptionNameGenerator.class},
                (proxy, method, values) -> null);
        for (String kind : new String[]{"Fork", "Join"}) {
            java.lang.Class<?> clazz = java.lang.Class.forName("org.eclipse.syson.diagram.common.view.nodes." + kind + "ActionNodeDescriptionProvider");
            Object provider = clazz.getConstructor(IColorProvider.class, IDescriptionNameGenerator.class).newInstance(colors, names);
            var method = clazz.getDeclaredMethod("createNodeStyleDescription"); method.setAccessible(true);
            var style = (org.eclipse.emf.ecore.EObject) method.invoke(provider);
            Object background = style.eGet(style.eClass().getEStructuralFeature("background"));
            String actual = ((org.eclipse.sirius.components.view.UserColor)background).getName();
            String expected = derived ? "actionGreen" : "black";
            if (!expected.equals(actual)) throw new AssertionError(kind + ": " + actual);
            System.out.println(kind + "\tbackground\t" + actual); count++;
        }
        System.out.println("PASS_SELECTED_PROVIDER_CASES=" + count);
    }
}
